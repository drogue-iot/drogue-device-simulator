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

    /// check if the claim is already claimed
    pub fn is_claimed(&self, check_claim: &Claim) -> bool {
        for (_, claims) in &self.claims {
            if claims.contains(check_claim) {
                return true;
            }
        }
        false
    }

    pub fn is_claimed_any(&self, check_claims: &[Claim]) -> bool {
        for check_claim in check_claims {
            if self.is_claimed(check_claim) {
                return true;
            }
        }
        false
    }
}
