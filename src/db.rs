use anyhow::Result as AnyResult;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::FromRow;
use log::info;

#[derive(FromRow, Debug)]
pub struct TaxOffice {
    pub name: String,
    pub zip: String,
    pub city: String,
    pub street: String,
    pub number: String,
}

pub async fn get_tax_office_query(zip: &String, name: &String) -> AnyResult<TaxOffice, anyhow::Error> {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite://data/db/tax_offices.db").await?;

    info!("using zip {} and name {}", zip, name);

    let row = sqlx::query_as::<_, TaxOffice>(
        "
SELECT
    ad.city,
    ad.street,
    ad.number,
    ad.zip,
    tao.name
FROM address ad
JOIN address_to_tax_office atto ON ad.id = atto.address_id
JOIN tax_office tao ON atto.tax_office_id = tao.id
WHERE
    zip = ?
    AND name = ?
UNION
SELECT
    pb.city,
    'Postfach' as street,
    pb.number,
    pb.zip,
    tao.name
FROM postbox pb
JOIN postbox_to_tax_office potto ON pb.id = potto.postbox_id
JOIN tax_office tao ON potto.tax_office_id = tao.id
WHERE
    zip = ?
    AND name = ?
"
    )
        .bind(zip.clone())
        .bind(name.clone())
        .bind(zip.clone())
        .bind(name.clone())
        .fetch_one(&pool).await?;

    Ok(row)
}