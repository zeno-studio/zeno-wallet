use graphql_client::{GraphQLQuery, Response}; // cargo add graphql-client reqwest
#[derive(GraphQLQuery)]
#[graphql(schema_path = "schema.graphql", query_path = "query.gql")]
struct NftMetadataQuery; // e.g., query { nfts { metadataURI } }
let query = NftMetadataQuery::build_query(...);
let resp = reqwest::post("https://api.thegraph.com/subgraphs/name/uniswap/uniswap-v3")
    .json(&query).send().await?;
let uri = resp.data.nfts[0].metadata_uri; // 拉 IPFS/Arweave