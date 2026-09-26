require("@nomicfoundation/hardhat-toolbox");
require("dotenv").config();

const { PRIVATE_KEY, CELOSCAN_API_KEY } = process.env;

// Required environment variables per deployable network. Local development
// (hardhat / localhost) stays usable without any of these.
const REQUIRED_ENV = {
  alfajores: ["PRIVATE_KEY", "ALFAJORES_RPC_URL"],
  celo: ["PRIVATE_KEY", "CELO_RPC_URL"],
};

// Canonical chain ids for every deployable network. The deployment manifest
// binds an address to one of these ids and must be rejected on any other chain.
const CHAIN_IDS = {
  alfajores: 44787,
  celo: 42220,
};

// Redact secrets so private keys and full RPC URLs are never printed.
function redact(value) {
  if (!value) return "<unset>";
  const str = String(value);
  if (str.length <= 8) return "<redacted>";
  return `${str.slice(0, 4)}…${str.slice(-4)}`;
}

function redactUrl(url) {
  if (!url) return "<unset>";
  try {
    const parsed = new URL(url);
    return `${parsed.protocol}//${parsed.host}/<redacted>`;
  } catch {
    return "<redacted>";
  }
}

// Fail closed: refuse to configure a production network when required
// configuration is absent, before any signing can occur.
function requireEnv(network) {
  const missing = REQUIRED_ENV[network].filter((key) => !process.env[key]);
  if (missing.length > 0) {
    throw new Error(
      `Refusing to configure network "${network}": missing required environment variable(s): ${missing.join(
        ", "
      )}. Set them in your .env before deploying to ${network}.`
    );
  }
}

function networkConfig(network, defaultUrl, chainId) {
  requireEnv(network);
  const url = process.env[`${network.toUpperCase()}_RPC_URL`] || defaultUrl;
  return {
    url,
    chainId,
    accounts: [PRIVATE_KEY],
  };
}

/** @type import('hardhat/config').HardhatUserConfig */
module.exports = {
  solidity: {
    version: "0.8.20",
    settings: {
      viaIR: true,
      optimizer: {
        enabled: true,
        runs: 200,
      },
    },
  },
  networks: {
    alfajores: networkConfig(
      "alfajores",
      "https://alfajores-forno.celo-testnet.org",
      CHAIN_IDS.alfajores
    ),
    celo: networkConfig("celo", "https://forno.celo.org", CHAIN_IDS.celo),
  },
  etherscan: {
    apiKey: {
      alfajores: CELOSCAN_API_KEY || "",
      celo: CELOSCAN_API_KEY || "",
    },
    customChains: [
      {
        network: "alfajores",
        chainId: CHAIN_IDS.alfajores,
        urls: {
          apiURL: "https://api-alfajores.celoscan.io/api",
          browserURL: "https://alfajores.celoscan.io",
        },
      },
      {
        network: "celo",
        chainId: CHAIN_IDS.celo,
        urls: {
          apiURL: "https://api.celoscan.io/api",
          browserURL: "https://celoscan.io",
        },
      },
    ],
  },
};

module.exports.redact = redact;
module.exports.redactUrl = redactUrl;
module.exports.CHAIN_IDS = CHAIN_IDS;
