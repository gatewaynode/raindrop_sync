use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};

use crate::api::RaindropClient;
use crate::models::{ApiRaindrop, Bookmark};

pub async fn sync(client: &RaindropClient, output_path: &Path) -> Result<usize> {
    let (raindrops, collections) = tokio::try_join!(
        client.get_all_raindrops(),
        client.get_collections(),
    )?;

    let collection_names: HashMap<i64, String> = collections
        .into_iter()
        .map(|c| (c.id, c.title))
        .collect();

    let bookmarks = map_bookmarks(raindrops, &collection_names);
    let count = bookmarks.len();

    let json = serde_json::to_string_pretty(&bookmarks).context("failed to serialize bookmarks")?;
    std::fs::write(output_path, json).context("failed to write bookmarks.json")?;

    Ok(count)
}

fn map_bookmarks(raindrops: Vec<ApiRaindrop>, collection_names: &HashMap<i64, String>) -> Vec<Bookmark> {
    raindrops
        .into_iter()
        .map(|r| {
            let name = r
                .collection
                .as_ref()
                .and_then(|c| collection_names.get(&c.id))
                .cloned()
                .unwrap_or_else(|| "Unsorted".to_string());
            Bookmark::from_api(r, &name)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::CollectionRef;

    fn make_raindrop(id: i64, collection_id: Option<i64>) -> ApiRaindrop {
        ApiRaindrop {
            id,
            title: Some(format!("Bookmark {id}")),
            link: format!("https://example.com/{id}"),
            tags: Some(vec!["tag".to_string()]),
            collection: collection_id.map(|id| CollectionRef { id }),
            created: Some("2024-01-01T00:00:00Z".to_string()),
            last_update: Some("2024-06-01T00:00:00Z".to_string()),
            excerpt: None,
            kind: Some("link".to_string()),
        }
    }

    #[test]
    fn test_map_bookmarks_resolves_collection_name() {
        let mut names = HashMap::new();
        names.insert(10, "Programming".to_string());

        let raindrops = vec![make_raindrop(1, Some(10))];
        let bookmarks = map_bookmarks(raindrops, &names);

        assert_eq!(bookmarks[0].collection, "Programming");
        assert_eq!(bookmarks[0].collection_id, 10);
    }

    #[test]
    fn test_map_bookmarks_falls_back_to_unsorted() {
        let raindrops = vec![make_raindrop(1, None)];
        let bookmarks = map_bookmarks(raindrops, &HashMap::new());

        assert_eq!(bookmarks[0].collection, "Unsorted");
        assert_eq!(bookmarks[0].collection_id, -1);
    }

    #[test]
    fn test_map_bookmarks_unknown_collection_id_falls_back_to_unsorted() {
        let raindrops = vec![make_raindrop(1, Some(999))];
        let bookmarks = map_bookmarks(raindrops, &HashMap::new());

        assert_eq!(bookmarks[0].collection, "Unsorted");
    }

    #[test]
    fn test_map_bookmarks_preserves_all_items() {
        let mut names = HashMap::new();
        names.insert(1, "A".to_string());
        names.insert(2, "B".to_string());

        let raindrops = vec![
            make_raindrop(10, Some(1)),
            make_raindrop(20, Some(2)),
            make_raindrop(30, None),
        ];
        let bookmarks = map_bookmarks(raindrops, &names);

        assert_eq!(bookmarks.len(), 3);
        assert_eq!(bookmarks[0].collection, "A");
        assert_eq!(bookmarks[1].collection, "B");
        assert_eq!(bookmarks[2].collection, "Unsorted");
    }
}
