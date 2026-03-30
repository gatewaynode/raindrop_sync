use serde::{Deserialize, Serialize};

/// Generic API list/items response wrapper
#[derive(Debug, Deserialize)]
pub struct ApiListResponse<T> {
    pub items: Vec<T>,
}

pub use ApiListResponse as ApiItemsResponse;

/// Raindrop collection reference embedded in a raindrop
#[derive(Debug, Deserialize)]
pub struct CollectionRef {
    #[serde(rename = "$id")]
    pub id: i64,
}

/// A single raindrop as returned by the API
#[derive(Debug, Deserialize)]
pub struct ApiRaindrop {
    #[serde(rename = "_id")]
    pub id: i64,
    pub title: Option<String>,
    pub link: String,
    pub tags: Option<Vec<String>>,
    pub collection: Option<CollectionRef>,
    pub created: Option<String>,
    #[serde(rename = "lastUpdate")]
    pub last_update: Option<String>,
    pub excerpt: Option<String>,
    #[serde(rename = "type")]
    pub kind: Option<String>,
}

/// A collection as returned by the API
#[derive(Debug, Deserialize)]
pub struct ApiCollection {
    #[serde(rename = "_id")]
    pub id: i64,
    pub title: String,
}

/// Output bookmark written to bookmarks.json
#[derive(Debug, Serialize)]
pub struct Bookmark {
    pub id: i64,
    pub title: String,
    pub link: String,
    pub tags: Vec<String>,
    pub collection_id: i64,
    pub collection: String,
    pub created: String,
    pub last_update: String,
    pub excerpt: String,
    #[serde(rename = "type")]
    pub kind: String,
}

impl Bookmark {
    pub fn from_api(r: ApiRaindrop, collection_name: &str) -> Self {
        let collection_id = r.collection.as_ref().map(|c| c.id).unwrap_or(-1);
        Bookmark {
            id: r.id,
            title: r.title.unwrap_or_default(),
            link: r.link,
            tags: r.tags.unwrap_or_default(),
            collection_id,
            collection: collection_name.to_string(),
            created: r.created.unwrap_or_default(),
            last_update: r.last_update.unwrap_or_default(),
            excerpt: r.excerpt.unwrap_or_default(),
            kind: r.kind.unwrap_or_else(|| "link".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bookmark_from_api_maps_fields() {
        let api = ApiRaindrop {
            id: 42,
            title: Some("Test".to_string()),
            link: "https://example.com".to_string(),
            tags: Some(vec!["a".to_string(), "b".to_string()]),
            collection: Some(CollectionRef { id: 7 }),
            created: Some("2024-01-01T00:00:00Z".to_string()),
            last_update: Some("2024-06-01T00:00:00Z".to_string()),
            excerpt: Some("A test bookmark".to_string()),
            kind: Some("link".to_string()),
        };

        let bm = Bookmark::from_api(api, "Dev");

        assert_eq!(bm.id, 42);
        assert_eq!(bm.title, "Test");
        assert_eq!(bm.link, "https://example.com");
        assert_eq!(bm.tags, vec!["a", "b"]);
        assert_eq!(bm.collection_id, 7);
        assert_eq!(bm.collection, "Dev");
        assert_eq!(bm.kind, "link");
    }

    #[test]
    fn test_bookmark_serializes_to_expected_json_shape() {
        let api = ApiRaindrop {
            id: 1,
            title: Some("Example".to_string()),
            link: "https://example.com".to_string(),
            tags: Some(vec!["rust".to_string()]),
            collection: Some(CollectionRef { id: 5 }),
            created: Some("2024-01-01T00:00:00Z".to_string()),
            last_update: Some("2024-06-01T00:00:00Z".to_string()),
            excerpt: Some("An example".to_string()),
            kind: Some("link".to_string()),
        };

        let bm = Bookmark::from_api(api, "Dev");
        let json: serde_json::Value = serde_json::to_value(&bm).unwrap();

        assert_eq!(json["id"], 1);
        assert_eq!(json["title"], "Example");
        assert_eq!(json["link"], "https://example.com");
        assert_eq!(json["tags"][0], "rust");
        assert_eq!(json["collection_id"], 5);
        assert_eq!(json["collection"], "Dev");
        assert_eq!(json["type"], "link");
        // field must be "type" in output, not "kind"
        assert!(json.get("kind").is_none());
    }

    #[test]
    fn test_api_raindrop_deserializes_from_api_json() {
        let json = r#"{
            "_id": 99,
            "title": "Rust Book",
            "link": "https://doc.rust-lang.org",
            "tags": ["rust"],
            "collection": {"$id": 3},
            "created": "2024-01-01T00:00:00Z",
            "lastUpdate": "2024-06-01T00:00:00Z",
            "excerpt": "The book",
            "type": "link"
        }"#;

        let r: ApiRaindrop = serde_json::from_str(json).unwrap();

        assert_eq!(r.id, 99);
        assert_eq!(r.title.unwrap(), "Rust Book");
        assert_eq!(r.collection.unwrap().id, 3);
        assert_eq!(r.last_update.unwrap(), "2024-06-01T00:00:00Z");
    }

    #[test]
    fn test_bookmark_from_api_uses_defaults_for_missing_fields() {
        let api = ApiRaindrop {
            id: 1,
            title: None,
            link: "https://example.com".to_string(),
            tags: None,
            collection: None,
            created: None,
            last_update: None,
            excerpt: None,
            kind: None,
        };

        let bm = Bookmark::from_api(api, "Unknown");

        assert_eq!(bm.title, "");
        assert!(bm.tags.is_empty());
        assert_eq!(bm.collection_id, -1);
        assert_eq!(bm.kind, "link");
    }
}
