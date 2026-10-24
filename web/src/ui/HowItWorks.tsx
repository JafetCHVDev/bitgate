import { CONFIG } from "../config";
import { ExtLink } from "./components";
import { useLang } from "./state";

export function HowItWorks() {
  const { t } = useLang();
  return (
    <div className="page-narrow how">
      <h1>{t("Cómo funciona", "How it works")}</h1>
      <p className="lede">
        {t(
          "En América Latina se cambia Bitcoin por dólares de persona a persona, en grupos de chat y mercados P2P. Siempre alguien tiene que enviar primero y confiar en el otro, o ambos confían en una plataforma que retiene los fondos. BitGate elimina esa confianza: el contrato solo suelta el USDC cuando Bitcoin mismo lo demuestra.",
          "Across Latin America people trade Bitcoin for dollars peer to peer, in chat groups and P2P marketplaces. Someone always has to send first and trust the other, or both trust a platform that holds the funds. BitGate removes that trust: the contract only lets go of the USDC when Bitcoin itself proves it should.",
        )}
      </p>

      <div className="flow" role="img" aria-label={t("Flujo de un trade", "Flow of a trade")}>
        <div className="lane">
          <div className="lane-title">{t("Vendedor", "Seller")}</div>
          <div className="lane-step">1. create_offer</div>
          <div className="lane-note">{t("bloquea USDC + su dirección BTC + precio", "locks USDC + BTC address + price")}</div>
        </div>
        <div className="lane">
          <div className="lane-title">{t("Comprador", "Buyer")}</div>
          <div className="lane-step">2. take</div>
          <div className="lane-note">{t("reserva un monto, deja un bono", "reserves an amount, posts a bond")}</div>
          <div className="lane-step btc">3. {t("paga BTC", "pays BTC")}</div>
          <div className="lane-note">{t("en Bitcoin, a la dirección del vendedor", "on Bitcoin, to the seller's address")}</div>
        </div>
        <div className="lane">
          <div className="lane-title">{t("Cualquiera", "Anyone")}</div>
          <div className="lane-step">4. claim(proof)</div>
          <div className="lane-note">{t("prueba del relayer de Writz o de Esplora", "proof from the Writz relayer or Esplora")}</div>
        </div>
        <div className="lane accent">
          <div className="lane-title">BitGate escrow</div>
          <div className="lane-step">→ bitcoin-spv.verify_transaction</div>
          <div className="lane-note">{t("llamada cross-contract al light client de Writz", "cross-contract call to the Writz light client")}</div>
          <div className="lane-step">✓ {t("script, monto, frescura, txid único", "script, amount, freshness, unique txid")}</div>
          <div className="lane-step ok">5. USDC → {t("comprador", "buyer")}</div>
        </div>
      </div>

      <h2>{t("Lo que el contrato garantiza", "What the contract guarantees")}</h2>
      <ul className="guarantees">
        <li>
          <strong>{t("Inclusión y profundidad", "Inclusion and depth")}</strong>{" "}
          {t(
            "las prueba el light client de Writz, que guarda headers de Bitcoin en Stellar y verifica proof-of-work, dificultad y la cadena con más trabajo.",
            "are proven by the Writz light client, which stores Bitcoin headers on Stellar and checks proof-of-work, difficulty and the most-work chain.",
          )}
        </li>
        <li>
          <strong>{t("Destinatario y monto", "Recipient and amount")}</strong>{" "}
          {t(
            "los lee el escrow de los bytes probados, con el parser on-chain de Writz: el pago debe ir al script del vendedor y cubrir lo pactado.",
            "are read by the escrow from the proven bytes, with Writz's on-chain parser: the payment must go to the seller's script and cover the agreed amount.",
          )}
        </li>
        <li>
          <strong>{t("Frescura", "Freshness")}</strong>{" "}
          {t(
            "al reservar se guarda el tip del light client; solo vale un pago minado después. Un pago viejo a la misma dirección no sirve.",
            "the light client tip is recorded at reservation; only a payment mined after it counts. An old payment to the same address does not work.",
          )}
        </li>
        <li>
          <strong>{t("Sin repetición", "No replay")}</strong>{" "}
          {t("cada txid de Bitcoin liquida un solo trade, en todas las ofertas.", "each Bitcoin txid settles one trade only, across all offers.")}
        </li>
        <li>
          <strong>{t("Sin bloqueos gratis", "No free griefing")}</strong>{" "}
          {t(
            "una reserva a la vez por oferta, con bono: si el comprador no paga, el bono va al vendedor y la oferta se libera sola.",
            "one reservation per offer at a time, with a bond: if the buyer does not pay, the bond goes to the seller and the offer frees itself.",
          )}
        </li>
        <li>
          <strong>{t("Sin intermediarios", "No middlemen")}</strong>{" "}
          {t(
            "la prueba la puede enviar cualquiera y el USDC siempre va al comprador; el relayer de Writz es opcional porque la app sabe armar la prueba desde Esplora.",
            "anyone can submit the proof and the USDC always goes to the buyer; the Writz relayer is optional because the app can build the proof from Esplora.",
          )}
        </li>
      </ul>

      <h2>{t("Por qué Stellar", "Why Stellar")}</h2>
      <p>
        {t(
          "El USDC de Circle es nativo en Stellar, las comisiones son de fracciones de centavo y Soroban permite que un contrato llame en vivo al light client de Bitcoin de Writz. Sin esa llamada cross-contract no hay producto: no es una app genérica portada a Stellar.",
          "Circle's USDC is native on Stellar, fees are fractions of a cent, and Soroban lets a contract call the Writz Bitcoin light client live. Without that cross-contract call there is no product: this is not a generic app ported to Stellar.",
        )}
      </p>

      <h2>{t("Integración con Writz", "Writz integration")}</h2>
      <ul className="guarantees">
        <li>
          {t("Llama a", "Calls")} <span className="mono">verify_transaction</span> {t("y", "and")} <span className="mono">get_best_tip</span>{" "}
          {t("del", "of the")} <span className="mono">bitcoin-spv</span>{" "}
          <ExtLink href={CONFIG.explorer.stellarContract(CONFIG.spvId)}>{t("desplegado en testnet", "deployed on testnet")}</ExtLink>.
        </li>
        <li>
          {t(
            "Usa el crate oficial writz-spv-client (cliente tipado, errores y parser) fijado a un commit, y traduce los errores de Writz a errores propios.",
            "Uses the official writz-spv-client crate (typed client, errors and parser) pinned to a commit, and maps Writz errors to its own.",
          )}
        </li>
        <li>
          {t(
            "Sus tests corren contra el WASM exacto que Writz desplegó, con headers reales de signet. Nada del lado Bitcoin está simulado.",
            "Its tests run against the exact WASM Writz deployed, with real signet headers. Nothing on the Bitcoin side is mocked.",
          )}
        </li>
        <li>
          {t(
            "Usa el relayer de Writz para las pruebas y, si no responde, las arma desde Esplora en el navegador (verificado byte a byte contra el relayer).",
            "Uses the Writz relayer for proofs and, if it is down, builds them from Esplora in the browser (checked byte for byte against the relayer).",
          )}
        </li>
        <li>
          {t(
            "Muestra el precio de Reflector, el mismo oráculo de Writz Lend, para comparar cada oferta con el mercado.",
            "Shows the Reflector price, the same oracle Writz Lend reads, to compare each offer with the market.",
          )}
        </li>
      </ul>

      <h2>{t("Siguiente", "Next")}</h2>
      <ul className="guarantees">
        <li>{t("Dirección inversa: vendedores de BTC que cobran en USDC.", "Reverse direction: BTC sellers paid in USDC.")}</li>
        <li>{t("Retiro a colones con un anchor SEP-24.", "Cash-out to colones through a SEP-24 anchor.")}</li>
        <li>
          {t(
            "Validar la demanda que Writz pide antes de construir Dark Swap, con trades reales en vez de encuestas.",
            "Validate the demand Writz asks for before building Dark Swap, with real trades instead of surveys.",
          )}
        </li>
        <li>{t("Mainnet cuando el light client de Writz llegue a mainnet.", "Mainnet once the Writz light client reaches mainnet.")}</li>
      </ul>

      <p className="muted small">
        {t("Solo testnet y signet. No auditado. ", "Testnet and signet only. Not audited. ")}
        <ExtLink href={CONFIG.repo}>{t("Código fuente", "Source code")}</ExtLink>
      </p>
    </div>
  );
}
