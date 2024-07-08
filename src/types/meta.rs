#[derive(Clone, PartialEq, Debug,)]
pub enum Meta {
    Spot {
        name: String,
        quote: SpotAssetMeta,
        base: SpotAssetMeta,
    },
    Perp {
        name: String,
        sz_decimals: u16,
    }
}

impl Meta {
    pub fn get_sz_decimals(&self) -> u16 {
        match self {
            Meta::Spot { quote, .. } => (*quote).sz_decimals,
            Meta::Perp { sz_decimals, .. } => *sz_decimals,
        }
    }

    pub fn is_spot(&self) -> bool {
        match self {
            Meta::Spot { .. } => true,
            _ => false,
        }
    }

    pub fn is_perp(&self) -> bool {
        match self {
            Meta::Perp { .. } => true,
            _ => false,
        }
    }

    pub fn get_name(&self) -> &String {
        match self {
            Meta::Spot { name, .. } => name,
            Meta::Perp { name, .. } => name,
        }
    }
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct SpotAssetMeta {
    pub sz_decimals: u16,
    pub wei_decimals: u16,
    pub name: String,
    pub index: u16,
}
