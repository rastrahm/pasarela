// Migrations are an early feature. Currently, they're nothing more than this
// single deploy script that's invoked from the CLI, injecting a provider
// configured from the workspace's Anchor.toml.

import * as anchor from "@coral-xyz/anchor";

// eslint-disable-next-line @typescript-eslint/no-require-imports
const devnet = require("../deploy/devnet.json") as {
  programId: string;
  cluster: string;
};

module.exports = async function (provider: anchor.AnchorProvider) {
  anchor.setProvider(provider);

  const programId = new anchor.web3.PublicKey(devnet.programId);
  const info = await provider.connection.getAccountInfo(programId);

  if (!info?.executable) {
    throw new Error(
      `Programa ${devnet.programId} no encontrado en ${provider.connection.rpcEndpoint}`,
    );
  }

  console.log(
    `payment-settlement desplegado: ${devnet.programId} (${devnet.cluster})`,
  );
};
