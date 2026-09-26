const hre = require("hardhat");

// Privileged (admin/emergency) actions must be replay-resistant and bound to
// the contract + network context. This helper tracks the per-action sequence
// (nonce) that privileged calls consume, so a replayed authorization is
// rejected and a nonce is only consumed once the action succeeds.
const PRIVILEGED_ACTIONS = ["setPolicy", "pause", "unpause", "emergencyWithdraw"];

function createNonceTracker(contractAddress, networkName) {
  const consumed = new Map();

  function context() {
    return { contract: contractAddress.toLowerCase(), network: networkName };
  }

  function nextSequence(action) {
    if (!PRIVILEGED_ACTIONS.includes(action)) {
      throw new Error(`Unknown privileged action: ${action}`);
    }
    return (consumed.get(action) || 0) + 1;
  }

  // Executes a privileged action with replay-resistant nonce semantics.
  // The nonce is only marked consumed after the transaction succeeds, so a
  // failed transaction can be retried with the same nonce.
  async function runPrivileged(action, send) {
    const sequence = nextSequence(action);
    const ctx = context();

    const tx = await send({ action, sequence, ...ctx });
    const receipt = await tx.wait();

    consumed.set(action, sequence);

    // Audit event exposes action type and sequence without secret material.
    console.log(
      JSON.stringify({
        event: "privileged-action",
        action,
        sequence,
        contract: ctx.contract,
        network: ctx.network,
        txHash: receipt.hash,
      })
    );

    return receipt;
  }

  return { runPrivileged, nextSequence, context };
}

async function main() {
  const Factory = await hre.ethers.getContractFactory("PetChainRegistry");
  const registry = await Factory.deploy();
  await registry.waitForDeployment();

  const address = await registry.getAddress();
  console.log(`PetChainRegistry deployed to: ${address}`);
  console.log(`Network: ${hre.network.name}`);

  // Bind privileged authorization to this contract + network context.
  const nonces = createNonceTracker(address, hre.network.name);
  console.log(
    `Privileged nonce context: ${JSON.stringify(nonces.context())}`
  );

  return address;
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
