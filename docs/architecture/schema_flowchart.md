erDiagram
TRADITIONS {
uuid id PK
text name "UNIQUE"
}

    WORKS {
        uuid id PK
        uuid tradition_id FK
        text slug "UNIQUE"
        text title
    }

    EDITIONS {
        uuid id PK
        uuid work_id FK
        text name
        varchar language_code
        boolean is_source
    }

    NODES {
        uuid id PK
        uuid work_id FK
        ltree path "UNIQUE (The Structural Overlay)"
        varchar node_type "e.g., book, chapter, sura"
        integer start_index "Range Start"
        integer end_index "Range End"
    }

    NODE_ALIASES {
        uuid id PK
        uuid node_id FK
        text alias "UNIQUE with node_id"
        boolean is_canonical
    }

    TEXTS {
        uuid id PK
        uuid edition_id FK
        integer absolute_index "The Universal Sequence"
        text body_text
    }

    %% Relationships
    TRADITIONS ||--o{ WORKS : "categorizes"
    WORKS ||--o{ EDITIONS : "has translations"
    WORKS ||--o{ NODES : "defines structural maps for"
    NODES ||--o{ NODE_ALIASES : "is referenced by"
    EDITIONS ||--o{ TEXTS : "contains"

    %% Conceptual link (Not a strict Foreign Key, but the mathematical bridge)
    NODES }o..o{ TEXTS : "Maps start/end bounds to absolute_index"