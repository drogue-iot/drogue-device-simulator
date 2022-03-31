use std::collections::HashMap;
use std::ops::Deref;

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum Claim {
    Channel {
        channel: String,
    },
    Feature {
        channel: String,
        feature: String,
    },
    Property {
        channel: String,
        feature: String,
        property: String,
    },
}

impl Claim {
    pub fn overlap(&self, other: &Self) -> bool {
        match self {
            Claim::Channel { channel } => match other {
                Claim::Channel {
                    channel: check_channel,
                }
                | Claim::Feature {
                    channel: check_channel,
                    ..
                }
                | Claim::Property {
                    channel: check_channel,
                    ..
                } => {
                    if channel == check_channel {
                        return true;
                    }
                }
            },
            Claim::Feature { channel, feature } => match other {
                Claim::Channel {
                    channel: check_channel,
                } => {
                    if channel == check_channel {
                        return true;
                    }
                }
                Claim::Feature {
                    channel: check_channel,
                    feature: check_feature,
                }
                | Claim::Property {
                    channel: check_channel,
                    feature: check_feature,
                    ..
                } => {
                    if channel == check_channel && feature == check_feature {
                        return true;
                    }
                }
            },
            Claim::Property {
                channel,
                feature,
                property,
            } => match other {
                Claim::Channel {
                    channel: check_channel,
                } => {
                    if channel == check_channel {
                        return true;
                    }
                }
                Claim::Feature {
                    channel: check_channel,
                    feature: check_feature,
                } => {
                    if channel == check_channel && feature == check_feature {
                        return true;
                    }
                }
                Claim::Property {
                    channel: check_channel,
                    feature: check_feature,
                    property: check_property,
                } => {
                    if channel == check_channel
                        && feature == check_feature
                        && property == check_property
                    {
                        return true;
                    }
                }
            },
        }

        false
    }
}

#[derive(Clone, Debug, Default)]
pub struct Claims {
    claims: HashMap<String, Vec<Claim>>,
}

impl Deref for Claims {
    type Target = HashMap<String, Vec<Claim>>;

    fn deref(&self) -> &Self::Target {
        &self.claims
    }
}

impl Claims {
    pub fn insert(&mut self, id: String, claims: Vec<Claim>) {
        self.claims.insert(id, claims);
    }

    pub fn remove(&mut self, id: &str) {
        self.claims.remove(id);
    }

    fn contains(&self, check_claim: &Claim, exclude: Option<&str>) -> bool {
        for (id, claims) in &self.claims {
            if let Some(exclude) = exclude {
                if exclude == id {
                    continue;
                }
            }
            for claim in claims {
                if claim.overlap(check_claim) {
                    return true;
                }
            }
        }
        false
    }

    pub fn is_claimed_any(&self, check_claims: Vec<Claim>, exclude: Option<&str>) -> bool {
        for check_claim in check_claims {
            if self.contains(&check_claim, exclude) {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::simulator::simulations::*;

    #[test]
    fn test_claims_conflict_1() {
        let mut state = Claims::default();
        state.insert(
            "sim1".to_string(),
            SingleTarget::new("state", "feature", "property").claims(),
        );

        let claims = SingleTarget::new("state", "feature", "property").claims();
        assert!(state.is_claimed_any(claims.clone(), None));
        assert!(!state.is_claimed_any(claims, Some("sim1")));

        let claims = FeatureTarget::new("state", "feature").claims();
        assert!(state.is_claimed_any(claims.clone(), None));
        assert!(!state.is_claimed_any(claims, Some("sim1")));
    }

    #[test]
    fn test_claims_conflict_2() {
        let mut state = Claims::default();
        state.insert(
            "sim1".to_string(),
            FeatureTarget::new("state", "feature").claims(),
        );

        let claims = SingleTarget::new("state", "feature", "property").claims();
        assert!(state.is_claimed_any(claims.clone(), None));
        assert!(!state.is_claimed_any(claims, Some("sim1")));

        let claims = FeatureTarget::new("state", "feature").claims();
        assert!(state.is_claimed_any(claims.clone(), None));
        assert!(!state.is_claimed_any(claims, Some("sim1")));
    }
}
