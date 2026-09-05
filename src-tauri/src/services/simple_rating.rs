//! Project-local simple Rating based on reliable K/D/A, damage and round totals.
use crate::models::demo::DemoRating;

pub const MODEL_VERSION: &str = "lb-rating-2.0";

fn clamp(value: f64, low: f64, high: f64) -> f64 {
    value.max(low).min(high)
}

fn round4(value: f64) -> f64 {
    (value * 10_000.0).round() / 10_000.0
}

pub fn calculate(
    kills: Option<u32>,
    deaths: Option<u32>,
    assists: Option<u32>,
    damage: Option<u32>,
    rounds: Option<u32>,
) -> Option<DemoRating> {
    let (kills, deaths, assists, damage, rounds) = (kills?, deaths?, assists?, damage?, rounds?);
    if rounds == 0 {
        return None;
    }
    let rounds = rounds as f64;
    let damage_component = clamp((damage as f64 / rounds) / 82.0, 0.0, 3.0);
    let kill_component = clamp((kills as f64 / rounds) / 0.70, 0.0, 2.5);
    let deaths_per_round = clamp(deaths as f64 / rounds, 0.0, 1.0);
    let survival_component = clamp((1.0 - deaths_per_round) / 0.68, 0.0, 2.5);
    let assist_component = clamp((assists as f64 / rounds) / 0.20, 0.0, 2.5);
    let rating = clamp(
        0.55 * damage_component
            + 0.20 * kill_component
            + 0.15 * survival_component
            + 0.10 * assist_component,
        0.0,
        3.0,
    );
    Some(DemoRating {
        model_version: MODEL_VERSION.into(),
        kill_component: round4(kill_component),
        damage_component: round4(damage_component),
        survival_component: round4(survival_component),
        assist_component: round4(assist_component),
        rating: round4(rating),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_vectors_match_lb_rating_2() {
        let average = calculate(Some(14), Some(6), Some(4), Some(1640), Some(20)).unwrap();
        assert_eq!(average.model_version, MODEL_VERSION);
        assert_eq!(average.kill_component, 1.0);
        assert_eq!(average.damage_component, 1.0);
        assert!((average.survival_component - 1.0294).abs() < 0.0001);
        assert_eq!(average.assist_component, 1.0);
        assert!((average.rating - 1.0044).abs() < 0.0001);

        let zero = calculate(Some(0), Some(20), Some(0), Some(0), Some(20)).unwrap();
        assert_eq!(zero.rating, 0.0);

        let dominant = calculate(Some(40), Some(0), Some(20), Some(5000), Some(20)).unwrap();
        assert_eq!(dominant.kill_component, 2.5);
        assert_eq!(dominant.damage_component, 3.0);
        assert!((dominant.survival_component - 1.4706).abs() < 0.0001);
        assert_eq!(dominant.assist_component, 2.5);
        assert!((dominant.rating - 2.6206).abs() < 0.0001);
    }

    #[test]
    fn high_volume_player_outranks_low_death_passive_player() {
        let passive = calculate(Some(14), Some(7), Some(0), Some(1421), Some(23)).unwrap();
        let high_impact = calculate(Some(24), Some(17), Some(8), Some(2765), Some(23)).unwrap();
        assert!(high_impact.rating > passive.rating);
    }

    #[test]
    fn missing_or_zero_round_inputs_are_unavailable() {
        assert!(calculate(Some(1), Some(1), Some(1), Some(100), Some(0)).is_none());
        assert!(calculate(None, Some(1), Some(1), Some(100), Some(1)).is_none());
        assert!(calculate(Some(1), None, Some(1), Some(100), Some(1)).is_none());
        assert!(calculate(Some(1), Some(1), None, Some(100), Some(1)).is_none());
        assert!(calculate(Some(1), Some(1), Some(1), None, Some(1)).is_none());
        assert!(calculate(Some(1), Some(1), Some(1), Some(100), None).is_none());
    }

    #[test]
    fn extreme_inputs_stay_in_range() {
        let rating = calculate(
            Some(u32::MAX),
            Some(0),
            Some(u32::MAX),
            Some(u32::MAX),
            Some(1),
        )
        .unwrap();
        assert!((0.0..=3.0).contains(&rating.rating));
    }
}
