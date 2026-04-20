use sqlx::postgres::PgPoolOptions;
use dotenvy::dotenv;
use std::env;
use sqlx::PgPool;

pub async fn setup_db() -> PgPool {
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgPoolOptions::new().connect(&db_url).await.unwrap()
}

/// Inserts a deterministic, rich dataset into the database for testing.
/// Includes John 17 (KJV/Greek), Psalm 91 (KJV), and Quran Sura 110.
pub async fn seed_universal_data(pool: &PgPool) {
    // ==========================================
    // 1. TAXONOMY LAYER (Traditions & Works)
    // ==========================================
    sqlx::query("INSERT INTO traditions (id, name) VALUES ('00000000-0000-0000-0000-000000000001', 'Abrahamic_Test') ON CONFLICT DO NOTHING").execute(pool).await.unwrap();
    sqlx::query("INSERT INTO works (id, tradition_id, slug, title) VALUES ('00000000-0000-0000-0000-000000000002', '00000000-0000-0000-0000-000000000001', 'bible_test', 'The Holy Bible') ON CONFLICT DO NOTHING").execute(pool).await.unwrap();
    sqlx::query("INSERT INTO works (id, tradition_id, slug, title) VALUES ('00000000-0000-0000-0000-000000000003', '00000000-0000-0000-0000-000000000001', 'quran_test', 'The Holy Quran') ON CONFLICT DO NOTHING").execute(pool).await.unwrap();

    // ==========================================
    // 2. EDITIONS LAYER (Translations)
    // ==========================================
    sqlx::query("INSERT INTO editions (id, work_id, name, language_code, is_source) VALUES ('00000000-0000-0000-0000-000000000010', '00000000-0000-0000-0000-000000000002', 'KJV', 'en', false), ('00000000-0000-0000-0000-000000000011', '00000000-0000-0000-0000-000000000002', 'SBLGNT', 'grc', true) ON CONFLICT DO NOTHING").execute(pool).await.unwrap();
    sqlx::query("INSERT INTO editions (id, work_id, name, language_code, is_source) VALUES ('00000000-0000-0000-0000-000000000012', '00000000-0000-0000-0000-000000000003', 'Clear_Quran', 'en', false) ON CONFLICT DO NOTHING").execute(pool).await.unwrap();

    // ==========================================
    // 3. STRUCTURAL SPINE LAYER (Nodes / Addresses)
    // ==========================================
    sqlx::query(r#"
        INSERT INTO nodes (id, work_id, path, node_type, sort_order) VALUES
        ('00000000-0000-0000-0000-000000000100', '00000000-0000-0000-0000-000000000002', 'bible_test.nt.john', 'book', 43.0),
        ('00000000-0000-0000-0000-000000000101', '00000000-0000-0000-0000-000000000002', 'bible_test.nt.john.17', 'chapter', 43.17),
        ('00000000-0000-0000-0000-000000000099', '00000000-0000-0000-0000-000000000002', 'bible_test.nt.john.17.2', 'verse', 43.17002),
        ('00000000-0000-0000-0000-000000000102', '00000000-0000-0000-0000-000000000002', 'bible_test.nt.john.17.3', 'verse', 43.17003),
        ('00000000-0000-0000-0000-000000000103', '00000000-0000-0000-0000-000000000002', 'bible_test.nt.john.17.4', 'verse', 43.17004),
        ('00000000-0000-0000-0000-000000000200', '00000000-0000-0000-0000-000000000002', 'bible_test.ot.psalms', 'book', 19.0),
        ('00000000-0000-0000-0000-000000000201', '00000000-0000-0000-0000-000000000002', 'bible_test.ot.psalms.91', 'chapter', 19.91),
        ('00000000-0000-0000-0000-000000000202', '00000000-0000-0000-0000-000000000002', 'bible_test.ot.psalms.91.1', 'verse', 19.91001),
        ('00000000-0000-0000-0000-000000000301', '00000000-0000-0000-0000-000000000003', 'quran_test.110', 'chapter', 110.0),
        ('00000000-0000-0000-0000-000000000302', '00000000-0000-0000-0000-000000000003', 'quran_test.110.1', 'verse', 110.001)
        ON CONFLICT DO NOTHING;
    "#).execute(pool).await.unwrap();

    // 3.5 Aliases
    sqlx::query("INSERT INTO node_aliases (node_id, alias, is_canonical) VALUES ('00000000-0000-0000-0000-000000000100', 'Jn', true) ON CONFLICT DO NOTHING").execute(pool).await.unwrap();

    // ==========================================
    // 4. CONTENT LAYER (The Actual Texts)
    // ==========================================
    sqlx::query(r#"
        INSERT INTO texts (id, node_id, edition_id, body_text) VALUES
        ('00000000-0000-0000-0000-000000001001', '00000000-0000-0000-0000-000000000102', '00000000-0000-0000-0000-000000000010', 'And this is life eternal, that they might know thee the only true God...'),
        ('00000000-0000-0000-0000-000000001002', '00000000-0000-0000-0000-000000000102', '00000000-0000-0000-0000-000000000011', 'αὕτη δέ ἐστιν ἡ αἰώνιος ζωή, ἵνα γινώσκωσιν σὲ τὸν μόνον ἀληθινὸν θεὸν...'),
        ('00000000-0000-0000-0000-000000001003', '00000000-0000-0000-0000-000000000202', '00000000-0000-0000-0000-000000000010', 'He that dwelleth in the secret place of the most High shall abide under the shadow of the Almighty.'),
        ('00000000-0000-0000-0000-000000001004', '00000000-0000-0000-0000-000000000302', '00000000-0000-0000-0000-000000000012', 'When the victory of Allah has come and the conquest,')
        ON CONFLICT DO NOTHING;
    "#).execute(pool).await.unwrap();
}
