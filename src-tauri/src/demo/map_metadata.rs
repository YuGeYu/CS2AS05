use serde::{Deserialize, Serialize};

pub const UPSTREAM_COMMIT: &str = "8961f5072fe4d42803dde68e8e71b3c90b216504";
pub const EMBEDDED_JSON: &str = include_str!("../../resources/demo-maps/cs2-map-metadata.json");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MapMetadata {
    pub name: String,
    pub position_x: f64,
    pub position_y: f64,
    pub scale: f64,
    pub threshold_z: f64,
    pub radar_size: u32,
    pub radar_asset: String,
    pub radar_sha256: String,
    pub lower_radar_asset: Option<String>,
    pub lower_radar_sha256: Option<String>,
    pub upstream_commit: String,
}

impl MapMetadata {
    pub fn scale_coordinate(&self, image_size: f64, x: f64, y: f64) -> (f64, f64) {
        (
            ((x - self.position_x) / self.scale) * image_size / f64::from(self.radar_size),
            ((self.position_y - y) / self.scale) * image_size / f64::from(self.radar_size),
        )
    }

    pub fn layer(&self, z: f64) -> &'static str {
        if z < self.threshold_z {
            "lower"
        } else {
            "upper"
        }
    }
}

pub fn embedded() -> Result<Vec<MapMetadata>, serde_json::Error> {
    serde_json::from_str(EMBEDDED_JSON)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn embedded_metadata_is_complete_and_unique() {
        let maps = embedded().unwrap();
        assert_eq!(maps.len(), 44);
        assert_eq!(
            maps.iter()
                .map(|map| &map.name)
                .collect::<BTreeSet<_>>()
                .len(),
            44
        );
        assert!(maps
            .iter()
            .all(|map| map.upstream_commit == UPSTREAM_COMMIT && map.radar_sha256.len() == 64));
    }

    #[test]
    fn coordinates_and_layers_match_upstream_contract() {
        let maps = embedded().unwrap();
        let dust2 = maps.iter().find(|map| map.name == "de_dust2").unwrap();
        assert_eq!(dust2.scale_coordinate(1024.0, -2476.0, 3239.0), (0.0, 0.0));
        let nuke = maps.iter().find(|map| map.name == "de_nuke").unwrap();
        assert_eq!((nuke.layer(-496.0), nuke.layer(-495.0)), ("lower", "upper"));
        let vertigo = maps.iter().find(|map| map.name == "de_vertigo").unwrap();
        assert_eq!(
            (vertigo.layer(11699.0), vertigo.layer(11700.0)),
            ("lower", "upper")
        );
        assert!(nuke.lower_radar_asset.is_some() && vertigo.lower_radar_asset.is_some());
    }
}
