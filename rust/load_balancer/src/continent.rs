#[derive(Debug, PartialEq)]
enum Continent {
    Africa,
    Antarctica,
    Asia,
    Europe,
    NorthAmerica,
    Oceania,
    SouthAmerica,
}

impl Continent {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "AF" => Some(Self::Africa),
            "AN" => Some(Self::Antarctica),
            "AS" => Some(Self::Asia),
            "EU" => Some(Self::Europe),
            "NA" => Some(Self::NorthAmerica),
            "OC" => Some(Self::Oceania),
            "SA" => Some(Self::SouthAmerica),
            _ => None,
        }
    }
}
