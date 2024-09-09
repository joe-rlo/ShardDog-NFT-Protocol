An implementing contract MUST include the following fields on-chain
spec: a string that MUST be formatted nft-n.n.n where "n.n.n" is replaced with the implemented version of this Metadata spec: for instance, "nft-2.0.0" to indicate NEP-177 version 2.0.0. This will allow consumers of the Non-Fungible Token to know which set of metadata features the contract supports.
name: the human-readable name of the contract.
symbol: the abbreviated symbol of the contract, like MOCHI or MV3
base_uri: Centralized gateway known to have reliable access to decentralized storage assets referenced by reference or media URLs. Can be used by other frontends for initial retrieval of assets, even if these frontends then replicate the data to their own decentralized nodes, which they are encouraged to do.

type NFTContractMetadata = {
  spec: string, // required, essentially a version like "nft-2.0.0", replacing "2.0.0" with the implemented version of NEP-177
  name: string, // required, ex. "Mochi Rising — Digital Edition" or "Metaverse 3"
  symbol: string, // required, ex. "MOCHI"
  icon: string|null, // Data URL
  base_uri: string|null, // Centralized gateway known to have reliable access to decentralized storage assets referenced by `reference` or `media` URLs
}