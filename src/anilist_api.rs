use reqwest::Client;
use serde_json::json;
// use serde::{Serialize, Deserialize};
//
// #[derive(Serialize, Deserialize, Debug)]
// struct Result{
//     id: usize,
// }

const SEARCH_QUERY: &str = "
query ($search: String!) { Page {
        media(search: $search, type: ANIME) {
            id
            title {
                native
                english
                romaji
            }
            siteUrl
            episodes
            type
        }
    }
}
";

// const QUERY: &str = "
//     query ($id: Int) {
//         Media (id: $id, type: ANIME) {
//             id
//             title {
//                 native
//                 english
//                 romaji
//             }
//             episodes
//             siteUrl
//             type
//         }
//     }
// ";
pub async fn browse(title: String) -> serde_json::Value {
    let query = SEARCH_QUERY;
    let json = json!({"query": query, "variables": {"search": title}});
    use_api(json).await
}

// pub async fn get_by_id(id: u32) -> serde_json::Value {
//     let query = QUERY;
//     let json = json!({"query": query, "variables": {"id": id}});
//     let result = use_api(json).await;
//
//     let anime = result["data"]["Media"].clone();
//
//     anime
// }

async fn use_api(body: serde_json::Value) -> serde_json::Value {
    let client = Client::new();
    // Define query and variables

    // Make HTTP post request
    let resp = client
        .post("https://graphql.anilist.co/")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .body(body.to_string())
        .send()
        .await
        .unwrap()
        .text()
        .await;
    // Get json
    let result: serde_json::Value = serde_json::from_str(&resp.unwrap()).unwrap();
    result
}
