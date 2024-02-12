import codegen from "@cosmwasm/ts-codegen";

codegen({
  contracts: [
    { name: "OmniflixMinterFactory", dir: "../contracts/factories/minter-factory/schema" },
    { name: "OmniflixOpenEditionMinterFactory", dir: "../contracts/factories/open-edition-minter-factory/schema" },
    { name: "OmniflixRoundWhitelistFactory", dir: "../contracts/factories/round-whitelist-factory/schema" },
    { name: "OmniflixRoundWhitelist", dir: "../contracts/whitelists/round-whitelist/schema" },
    { name: "OmniflixMinter", dir: "../contracts/minters/minter/schema" },
    { name: "OmniflixOpenEditionMinter", dir: "../contracts/minters/open-edition-minter/schema" },
    { name: "OmniflixOpenEditionMinterCopy", dir: "../contracts/minters/open-edition-minter copy/schema" },
  ],
  outPath: "./types/",
  options: {
    bundle: {
      bundleFile: "index.ts",
      scope: "contracts",
    },
    types: {
      enabled: true,
    },
    client: {
      enabled: true,
    },
    reactQuery: {
      enabled: false,
      optionalClient: false,
      version: "v3",
      mutations: false,
      queryKeys: false,
    },
    recoil: {
      enabled: false,
    },
    messageComposer: {
      enabled: true,
    },
  },
})
  .then(() => {
    console.log("Ts codegen success");
  })
