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
    sqlx::query(
        r#"
            INSERT INTO traditions (id, name) VALUES
            ('00000000-0000-0000-0000-000000000001', 'Abrahamic'),
            ('00000000-0000-0000-0000-000000000002', 'Vedic')
            ON CONFLICT (id) DO NOTHING;
            "#
    ).execute(pool).await.unwrap();

    sqlx::query(
        r#"
            INSERT INTO works (id, tradition_id, slug, title) VALUES
            ('00000000-0000-0000-0000-000000000010', '00000000-0000-0000-0000-000000000001', 'bible', 'The Holy Bible'),
            ('00000000-0000-0000-0000-000000000011', '00000000-0000-0000-0000-000000000001', 'tanakh', 'The Hebrew Bible'),
            ('00000000-0000-0000-0000-000000000012', '00000000-0000-0000-0000-000000000001', 'quran_hafs', 'The Quran (Hafs)'),
            ('00000000-0000-0000-0000-000000000013', '00000000-0000-0000-0000-000000000001', 'quran_warsh', 'The Quran (Warsh)'),
            ('00000000-0000-0000-0000-000000000014', '00000000-0000-0000-0000-000000000002', 'rigveda', 'The Rig Veda')
            ON CONFLICT (id) DO NOTHING;
            "#
    ).execute(pool).await.unwrap();
    // ==========================================
    // 2. EDITIONS LAYER (Translations)
    // ==========================================
    sqlx::query(
        r#"
            INSERT INTO editions (id, work_id, name, language_code, is_source) VALUES
            -- Bible Editions
            ('00000000-0000-0000-0000-000000000101', '00000000-0000-0000-0000-000000000010', 'KJV', 'en', false),
            ('00000000-0000-0000-0000-000000000102', '00000000-0000-0000-0000-000000000010', 'NIV', 'en', false),
            ('00000000-0000-0000-0000-000000000103', '00000000-0000-0000-0000-000000000010', 'LXX_Septuagint', 'grc', true),
            ('00000000-0000-0000-0000-000000000104', '00000000-0000-0000-0000-000000000010', 'SBLGNT', 'grc', true),

            -- Tanakh Editions
            ('00000000-0000-0000-0000-000000000111', '00000000-0000-0000-0000-000000000011', 'BHS_Hebrew', 'he', true),

            -- Quran Editions
            ('00000000-0000-0000-0000-000000000121', '00000000-0000-0000-0000-000000000012', 'Hafs_Arabic', 'ar', true),
            ('00000000-0000-0000-0000-000000000122', '00000000-0000-0000-0000-000000000012', 'Clear_Quran', 'en', false),
            ('00000000-0000-0000-0000-000000000131', '00000000-0000-0000-0000-000000000013', 'Warsh_Arabic', 'ar', true),
            ('00000000-0000-0000-0000-000000000132', '00000000-0000-0000-0000-000000000013', 'Rashad_Khalifa', 'en', false),

            -- Rig Veda Editions
            ('00000000-0000-0000-0000-000000000141', '00000000-0000-0000-0000-000000000014', 'Sanskrit_Original', 'sa', true),
            ('00000000-0000-0000-0000-000000000142', '00000000-0000-0000-0000-000000000014', 'Griffith_Translation', 'en', false)
            ON CONFLICT (id) DO NOTHING;
            "#
    ).execute(pool).await.unwrap();
    // ==========================================
    // 3. STRUCTURAL SPINE LAYER (Nodes / Addresses)
    // ==========================================
    sqlx::query(r#"
        INSERT INTO nodes (id, work_id, path, node_type, start_index, end_index) VALUES

        -- TEST CASE 1: BIBLE vs TANAKH (The Psalm 51 Title Shift)
        -- Bible Map: Combines indices 1000 and 1001 into a single unnumbered "title" node.
        ('00000000-0000-0000-0000-000000000A01', '00000000-0000-0000-0000-000000000010', 'bible.ot.psalms.51.title', 'superscription', 1000, 1001),
        ('00000000-0000-0000-0000-000000000A02', '00000000-0000-0000-0000-000000000010', 'bible.ot.psalms.51.1', 'verse', 1002, 1002),
        -- Tanakh Map: Counts the title lines as explicit verses 1 and 2!
        ('00000000-0000-0000-0000-000000000A03', '00000000-0000-0000-0000-000000000011', 'tanakh.ketuvim.psalms.51.1', 'verse', 1000, 1000),
        ('00000000-0000-0000-0000-000000000A04', '00000000-0000-0000-0000-000000000011', 'tanakh.ketuvim.psalms.51.2', 'verse', 1001, 1001),
        ('00000000-0000-0000-0000-000000000A05', '00000000-0000-0000-0000-000000000011', 'tanakh.ketuvim.psalms.51.3', 'verse', 1002, 1002),

        -- TEST CASE 2: QURAN HAFS vs WARSH (The Basmala Shift in Al-Fatiha)
        -- Hafs Map: The Basmala is explicitly Ayah 1.
        ('00000000-0000-0000-0000-000000000A06', '00000000-0000-0000-0000-000000000012', 'hafs.sura.1.1', 'ayah', 2000, 2000),
        ('00000000-0000-0000-0000-000000000A07', '00000000-0000-0000-0000-000000000012', 'hafs.sura.1.2', 'ayah', 2001, 2001),
        -- Warsh Map: The Basmala is an unnumbered title, "Alhamdulillah" becomes Ayah 1.
        ('00000000-0000-0000-0000-000000000A08', '00000000-0000-0000-0000-000000000013', 'warsh.sura.1.title', 'superscription', 2000, 2000),
        ('00000000-0000-0000-0000-000000000A09', '00000000-0000-0000-0000-000000000013', 'warsh.sura.1.1', 'ayah', 2001, 2001),

        -- TEST CASE 3: RIG VEDA (Overlapping Dual Hierarchies in the same Work)
        -- System A: Mandala (Theological)
        ('00000000-0000-0000-0000-000000000A10', '00000000-0000-0000-0000-000000000014', 'rigveda.mandala.1.sukta.1.mantra.1', 'sloka', 3000, 3000),
        -- System B: Ashtaka (Memorization)
        ('00000000-0000-0000-0000-000000000A11', '00000000-0000-0000-0000-000000000014', 'rigveda.ashtaka.1.adhyaya.1.varga.1.mantra.1', 'sloka', 3000, 3000),

        -- NT BIBLE PATH (For existing routing tests)
        ('00000000-0000-0000-0000-000000000A12', '00000000-0000-0000-0000-000000000010', 'bible.nt.john.17.3', 'verse', 4000, 4000)
        ON CONFLICT (id) DO NOTHING;
    "#).execute(pool).await.unwrap();

    // 3.5 Aliases
    sqlx::query("INSERT INTO node_aliases (node_id, alias, is_canonical) VALUES ('00000000-0000-0000-0000-000000000A12', 'Jn 17:3', true) ON CONFLICT DO NOTHING").execute(pool).await.unwrap();

    // ==========================================
    // 4. CONTENT LAYER (The Universal Absolute Sequence Texts)
    // ==========================================
    sqlx::query(r#"
        INSERT INTO texts (edition_id, absolute_index, body_text) VALUES

        -- BIBLE / TANAKH BASE (Indices 1000 - 1002)
        ('00000000-0000-0000-0000-000000000111', 1000, 'לַמְנַצֵּחַ מִזְמוֹר לְדָוִד׃'),
        ('00000000-0000-0000-0000-000000000111', 1001, 'בְּבוֹא־אֵלָיו נָתָן הַנָּבִיא...'),
        ('00000000-0000-0000-0000-000000000111', 1002, 'חָנֵּנִי אֱלֹהִים כְּחַסְדֶּךָ...'),

        ('00000000-0000-0000-0000-000000000101', 1000, 'To the chief Musician, A Psalm of David,'),
        ('00000000-0000-0000-0000-000000000101', 1001, 'when Nathan the prophet came unto him...'),
        ('00000000-0000-0000-0000-000000000101', 1002, 'Have mercy upon me, O God, according to thy lovingkindness...'),

        ('00000000-0000-0000-0000-000000000102', 1000, 'For the director of music. A psalm of David.'),
        ('00000000-0000-0000-0000-000000000102', 1001, 'When the prophet Nathan came to him...'),
        ('00000000-0000-0000-0000-000000000102', 1002, 'Have mercy on me, O God, according to your unfailing love...'),

        ('00000000-0000-0000-0000-000000000103', 1000, 'Εἰς τὸ τέλος· ψαλμὸς τῷ Δαυίδ,'),
        ('00000000-0000-0000-0000-000000000103', 1001, 'ἐν τῷ ἐλθεῖν πρὸς αὐτὸν Νάθαν τὸν προφήτην...'),
        ('00000000-0000-0000-0000-000000000103', 1002, 'Ἐλέησόν με, ὁ Θεός, κατὰ τὸ μέγα ἔλεός σου...'),

        -- QURAN BASE (Indices 2000 - 2001)
        ('00000000-0000-0000-0000-000000000121', 2000, 'بِسْمِ اللَّهِ الرَّحْمَٰنِ الرَّحِيمِ'),
        ('00000000-0000-0000-0000-000000000131', 2000, 'بِسْمِ اللَّهِ الرَّحْمَٰنِ الرَّحِيمِ'),
        ('00000000-0000-0000-0000-000000000122', 2000, 'In the name of Allah, the Entirely Merciful, the Especially Merciful.'),
        ('00000000-0000-0000-0000-000000000132', 2000, 'In the name of GOD, Most Gracious, Most Merciful.'),

        ('00000000-0000-0000-0000-000000000121', 2001, 'الْحَمْدُ لِلَّهِ رَبِّ الْعَالَمِينَ'),
        ('00000000-0000-0000-0000-000000000131', 2001, 'الْحَمْدُ لِلَّهِ رَبِّ الْعَالَمِينَ'),
        ('00000000-0000-0000-0000-000000000122', 2001, 'Praise be to Allah, Lord of the worlds.'),
        ('00000000-0000-0000-0000-000000000132', 2001, 'Praise be to GOD, Lord of the universe.'),

        -- RIG VEDA BASE (Index 3000)
        ('00000000-0000-0000-0000-000000000141', 3000, 'अग्निमीळे पुरोहितं यज्ञस्य देवमृत्विजम् । होतारं रत्नधातमम् ॥'),
        ('00000000-0000-0000-0000-000000000142', 3000, 'I laud Agni, the chosen Priest, God, minister of sacrifice, The hotar, lavishest of wealth.'),

        -- NT JOHN TEST (Index 4000)
        ('00000000-0000-0000-0000-000000000101', 4000, 'And this is life eternal...'),
        ('00000000-0000-0000-0000-000000000104', 4000, 'αὕτη δέ ἐστιν ἡ αἰώνιος ζωή...')

        ON CONFLICT DO NOTHING;
    "#).execute(pool).await.unwrap();
}
