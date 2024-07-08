#[derive(Clone, PartialEq, Debug,)]
pub enum Context {
    Spot {
        name: String,
        quote: SpotAssetContext,
        base: SpotAssetContext,
    },
    Perp {
        name: String,
        sz_decimals: u16,
    }
}

impl Context {
    pub fn get_sz_decimals(&self) -> u16 {
        match self {
            Context::Spot { quote, .. } => (*quote).sz_decimals,
            Context::Perp { sz_decimals, .. } => *sz_decimals,
        }
    }

    pub fn is_spot(&self) -> bool {
        match self {
            Context::Spot { .. } => true,
            _ => false,
        }
    }

    pub fn is_perp(&self) -> bool {
        match self {
            Context::Perp { .. } => true,
            _ => false,
        }
    }

    pub fn get_name(&self) -> &String {
        match self {
            Context::Spot { name, .. } => name,
            Context::Perp { name, .. } => name,
        }
    }
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct SpotAssetContext {
    pub sz_decimals: u16,
    pub wei_decimals: u16,
    pub name: String,
    pub index: u16,
}
