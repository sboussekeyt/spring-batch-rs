/// Represents the supported database types for RDBC operations.
///
/// This enum allows users to specify which database backend to use
/// when configuring readers and writers through the unified builder API.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseType {
    /// PostgreSQL database
    Postgres,
    /// MySQL database
    MySql,
    /// SQLite database
    Sqlite,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_support_equality_comparison() {
        assert_eq!(DatabaseType::Postgres, DatabaseType::Postgres);
        assert_eq!(DatabaseType::MySql, DatabaseType::MySql);
        assert_eq!(DatabaseType::Sqlite, DatabaseType::Sqlite);
        assert_ne!(
            DatabaseType::Postgres,
            DatabaseType::MySql,
            "Postgres should differ from MySql"
        );
        assert_ne!(
            DatabaseType::MySql,
            DatabaseType::Sqlite,
            "MySql should differ from Sqlite"
        );
        assert_ne!(
            DatabaseType::Postgres,
            DatabaseType::Sqlite,
            "Postgres should differ from Sqlite"
        );
    }

    #[test]
    fn should_be_copyable() {
        let original = DatabaseType::Postgres;
        let copy = original; // Copy semantics, original still valid
        assert_eq!(original, copy);
    }

    #[test]
    fn should_be_cloneable() {
        let original = DatabaseType::MySql;
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn should_format_debug_output() {
        assert_eq!(format!("{:?}", DatabaseType::Postgres), "Postgres");
        assert_eq!(format!("{:?}", DatabaseType::MySql), "MySql");
        assert_eq!(format!("{:?}", DatabaseType::Sqlite), "Sqlite");
    }
}
