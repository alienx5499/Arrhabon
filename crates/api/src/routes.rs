//! Routes: each handler parses input, runs one app use case, and renders JSON.

use arrhabon_app::commands::{AttestVisa, Contribute, CreateVault, QuoteCrossChainFunding};
use arrhabon_app::ports::{SignedTx, SwapQuoter, VaultLedger, VisaOutcome};
use arrhabon_app::queries::{FunderView, RegistryView, UniversityProof, VaultView};
use arrhabon_domain::{Deadline, Usdc};
use std::time::Duration;

use axum::error_handling::HandleErrorLayer;
use axum::extract::{DefaultBodyLimit, Path, State};
use axum::http::{HeaderValue, Method, StatusCode, header};
use axum::routing::{get, post};
use axum::{BoxError, Json, Router};
use serde::Deserialize;
use serde_json::{Value, json};
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::dto;
use crate::error::{ApiError, Body, Query};
use crate::state::{AppState, Ports};

type Res = Result<Json<Value>, ApiError>;

/// Signed Solana transactions are at most 1,232 bytes; JSON bodies here stay far below this.
pub const BODY_LIMIT: usize = 16 * 1024;

/// `/tx/submit` waits for confirmation (about a blockhash lifetime, ~60 s) and is open to
/// anyone, so in-flight submissions are capped and each is cut off after this.
pub const SUBMIT_TIMEOUT: Duration = Duration::from_secs(75);
const SUBMIT_CONCURRENCY: usize = 16;

/// Longest accepted origin chain, asset id or refund address.
pub const MAX_FIELD_LEN: usize = 128;

pub fn router<P: Ports>(state: AppState<P>, web_origin: HeaderValue) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(web_origin)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::CONTENT_TYPE]);
    Router::new()
        .route("/health", get(health::<P>))
        .route("/registry", get(registry::<P>))
        .route("/vaults", post(create_vault::<P>))
        .route("/vaults/{id}", get(vault::<P>))
        .route("/vaults/{id}/contributions", post(contribute::<P>))
        .route("/students/{id}/vaults", get(student_vaults::<P>))
        .route("/funders/{id}", get(funder::<P>))
        .route("/universities/{id}/proof", get(university_proof::<P>))
        .route(
            "/tx/submit",
            post(submit::<P>).layer(
                ServiceBuilder::new()
                    .layer(HandleErrorLayer::new(submit_overloaded))
                    .timeout(SUBMIT_TIMEOUT)
                    .concurrency_limit(SUBMIT_CONCURRENCY),
            ),
        )
        .route("/demo/attest", post(demo_attest::<P>))
        .route("/funding/quote", post(quote::<P>))
        .route("/funding/status", get(funding_status::<P>))
        .fallback(not_found)
        .layer(DefaultBodyLimit::max(BODY_LIMIT))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn not_found() -> ApiError {
    ApiError::new(StatusCode::NOT_FOUND, "not_found", "no such route")
}

async fn submit_overloaded(e: BoxError) -> ApiError {
    match e.is::<tower::timeout::error::Elapsed>() {
        true => ApiError::new(
            StatusCode::GATEWAY_TIMEOUT,
            "timeout",
            "confirmation timed out; the transaction may still land: re-read state before retrying",
        ),
        false => ApiError::new(StatusCode::SERVICE_UNAVAILABLE, "unavailable", "try again"),
    }
}

async fn health<P: Ports>(State(s): State<AppState<P>>) -> Json<Value> {
    let chain = match s.ledger.get() {
        Some(_) => "ready",
        None => "not_initialised",
    };
    Json(json!({
        "status": "ok",
        "cluster": s.cluster,
        "chain": chain,
        "crank": s.crank,
        "demo_attestor": s.attestor.is_some(),
    }))
}

async fn registry<P: Ports>(State(s): State<AppState<P>>) -> Res {
    Ok(Json(dto::registry(&RegistryView::load(s.ledger()?).await?)))
}

async fn vault<P: Ports>(State(s): State<AppState<P>>, Path(id): Path<String>) -> Res {
    let view = VaultView::load(s.ledger()?, &s.clock, dto::account(&id)?).await?;
    Ok(Json(dto::vault_view(&view)))
}

async fn student_vaults<P: Ports>(State(s): State<AppState<P>>, Path(id): Path<String>) -> Res {
    let views = VaultView::for_student(s.ledger()?, &s.clock, dto::account(&id)?).await?;
    Ok(Json(views.iter().map(dto::vault_view).collect()))
}

async fn funder<P: Ports>(State(s): State<AppState<P>>, Path(id): Path<String>) -> Res {
    let view = FunderView::load(s.ledger()?, dto::account(&id)?).await?;
    Ok(Json(dto::funder_view(&view)))
}

async fn university_proof<P: Ports>(State(s): State<AppState<P>>, Path(id): Path<String>) -> Res {
    let proofs = UniversityProof::for_university(s.ledger()?, dto::account(&id)?).await?;
    Ok(Json(proofs.iter().map(dto::university_proof).collect()))
}

#[derive(Deserialize)]
struct CreateVaultBody {
    student: String,
    university: String,
    attestor: String,
    /// USDC micro-units.
    target: u64,
    /// Unix seconds.
    deadline: i64,
    nonce: u64,
}

async fn create_vault<P: Ports>(
    State(s): State<AppState<P>>,
    Body(b): Body<CreateVaultBody>,
) -> Res {
    let built = CreateVault {
        student: dto::account(&b.student)?,
        university: dto::account(&b.university)?,
        attestor: dto::account(&b.attestor)?,
        target: Usdc::new(b.target),
        deadline: Deadline::new(b.deadline),
        nonce: b.nonce,
    }
    .execute(s.ledger()?, &s.clock)
    .await?;
    let mut out = dto::unsigned_tx(&built.tx);
    out["vault"] = arrhabon_app::address::to_base58(built.vault).into();
    Ok(Json(out))
}

#[derive(Deserialize)]
struct ContributeBody {
    funder: String,
    /// USDC micro-units.
    amount: u64,
}

async fn contribute<P: Ports>(
    State(s): State<AppState<P>>,
    Path(id): Path<String>,
    Body(b): Body<ContributeBody>,
) -> Res {
    let tx = Contribute {
        vault: dto::account(&id)?,
        funder: dto::account(&b.funder)?,
        amount: Usdc::new(b.amount),
    }
    .execute(s.ledger()?, &s.clock)
    .await?;
    Ok(Json(dto::unsigned_tx(&tx)))
}

#[derive(Deserialize)]
struct SubmitBody {
    /// Base64 of the wallet-signed transaction.
    tx: String,
}

async fn submit<P: Ports>(State(s): State<AppState<P>>, Body(b): Body<SubmitBody>) -> Res {
    let tx = SignedTx(dto::decode_tx(&b.tx)?);
    let sig = s
        .ledger()?
        .submit(tx)
        .await
        .map_err(arrhabon_app::AppError::from)?;
    Ok(Json(json!({ "signature": sig.0 })))
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum Outcome {
    Issued,
    Denied,
}

#[derive(Deserialize)]
struct AttestBody {
    vault: String,
    outcome: Outcome,
}

/// DEMO ONLY: signs the visa outcome with the project's demo attestor key.
async fn demo_attest<P: Ports>(State(s): State<AppState<P>>, Body(b): Body<AttestBody>) -> Res {
    let signer = s.attestor.as_ref().ok_or_else(|| {
        ApiError::new(
            StatusCode::FORBIDDEN,
            "demo_attestor_disabled",
            "DEMO attestor is disabled; set DEMO_ATTESTOR=true to enable it",
        )
    })?;
    let vault = dto::account(&b.vault)?;
    let outcome = match b.outcome {
        Outcome::Issued => VisaOutcome::Issued,
        Outcome::Denied => VisaOutcome::Denied,
    };
    let vault_b58 = arrhabon_app::address::to_base58(vault);
    tracing::info!(vault = %vault_b58, ?outcome, "DEMO attestor signing visa outcome");
    let sig = AttestVisa { vault, outcome }
        .execute(s.ledger()?, signer, &s.clock)
        .await?;
    Ok(Json(json!({
        "demo": true,
        "notice": "DEMO attestor: signed with a demo-only key, not a real visa decision",
        "signature": sig.0,
    })))
}

#[derive(Deserialize)]
struct QuoteBody {
    vault: String,
    origin_chain: String,
    origin_asset: String,
    /// Smallest unit of the origin asset, as a decimal string (may exceed 2^53).
    origin_amount: String,
    funder: String,
    refund_to: String,
    slippage_bps: u16,
}

async fn quote<P: Ports>(State(s): State<AppState<P>>, Body(b): Body<QuoteBody>) -> Res {
    for (name, value) in [
        ("origin_chain", &b.origin_chain),
        ("origin_asset", &b.origin_asset),
        ("refund_to", &b.refund_to),
    ] {
        if value.len() > MAX_FIELD_LEN {
            return Err(ApiError::bad_request(format!(
                "{name} is longer than {MAX_FIELD_LEN} characters"
            )));
        }
    }
    let origin_amount = b
        .origin_amount
        .parse()
        .map_err(|_| ApiError::bad_request("origin_amount must be a decimal integer string"))?;
    let quote = QuoteCrossChainFunding {
        vault: dto::account(&b.vault)?,
        origin_chain: b.origin_chain,
        origin_asset: b.origin_asset,
        origin_amount,
        funder: dto::account(&b.funder)?,
        refund_to: b.refund_to,
        slippage_bps: b.slippage_bps,
    }
    .execute(s.ledger()?, &s.quoter, &s.clock)
    .await?;
    Ok(Json(dto::swap_quote(&quote)))
}

#[derive(Deserialize)]
struct StatusQuery {
    deposit_address: String,
}

async fn funding_status<P: Ports>(
    State(s): State<AppState<P>>,
    Query(q): Query<StatusQuery>,
) -> Res {
    let status = s
        .quoter
        .status(&q.deposit_address)
        .await
        .map_err(arrhabon_app::AppError::from)?;
    Ok(Json(
        json!({ "deposit_address": q.deposit_address, "status": dto::swap_status(status) }),
    ))
}
