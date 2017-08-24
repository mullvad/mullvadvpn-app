pub type CountryCode = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub latlong: [f64; 2],
    pub country: String,
    pub city: String,
}
