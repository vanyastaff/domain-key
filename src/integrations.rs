//! Framework/database integrations behind feature flags.
//!
//! This module wires `domain-key` types into common ecosystem crates:
//! - `sqlx` for database encoding/decoding
//! - `axum` for HTTP error responses
//! - `actix-web` for HTTP error responses

#[cfg(feature = "sqlx")]
mod sqlx_support {
    use core::convert::TryFrom;
    use std::io;

    use sqlx::decode::Decode;
    use sqlx::encode::{Encode, IsNull};
    use sqlx::error::BoxDynError;
    use sqlx::{Database, Type};

    use crate::{Id, IdDomain, IdParseError, Key, KeyDomain};
    #[cfg(feature = "ulid")]
    use crate::{Ulid, UlidDomain};
    #[cfg(feature = "uuid")]
    use crate::{Uuid, UuidDomain};

    impl<DB, D> Type<DB> for Key<D>
    where
        DB: Database,
        D: KeyDomain,
        String: Type<DB>,
    {
        fn type_info() -> DB::TypeInfo {
            <String as Type<DB>>::type_info()
        }

        fn compatible(ty: &DB::TypeInfo) -> bool {
            <String as Type<DB>>::compatible(ty)
        }
    }

    impl<'q, DB, D> Encode<'q, DB> for Key<D>
    where
        DB: Database,
        D: KeyDomain,
        String: Encode<'q, DB>,
    {
        fn encode_by_ref(
            &self,
            buf: &mut <DB as Database>::ArgumentBuffer<'q>,
        ) -> Result<IsNull, BoxDynError> {
            <String as Encode<'q, DB>>::encode(self.as_str().to_owned(), buf)
        }

        fn size_hint(&self) -> usize {
            self.len()
        }
    }

    impl<'r, DB, D> Decode<'r, DB> for Key<D>
    where
        DB: Database,
        D: KeyDomain,
        String: Decode<'r, DB>,
    {
        fn decode(value: <DB as Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
            let decoded = <String as Decode<'r, DB>>::decode(value)?;
            Key::from_string(decoded).map_err(|error| Box::new(error) as BoxDynError)
        }
    }

    impl<DB, D> Type<DB> for Id<D>
    where
        DB: Database,
        D: IdDomain,
        i64: Type<DB>,
    {
        fn type_info() -> DB::TypeInfo {
            <i64 as Type<DB>>::type_info()
        }

        fn compatible(ty: &DB::TypeInfo) -> bool {
            <i64 as Type<DB>>::compatible(ty)
        }
    }

    impl<'q, DB, D> Encode<'q, DB> for Id<D>
    where
        DB: Database,
        D: IdDomain,
        i64: Encode<'q, DB>,
    {
        fn encode_by_ref(
            &self,
            buf: &mut <DB as Database>::ArgumentBuffer<'q>,
        ) -> Result<IsNull, BoxDynError> {
            let value = i64::try_from(self.get()).map_err(|_| {
                Box::new(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Id value does not fit into signed BIGINT",
                )) as BoxDynError
            })?;

            value.encode_by_ref(buf)
        }

        fn size_hint(&self) -> usize {
            core::mem::size_of::<i64>()
        }
    }

    impl<'r, DB, D> Decode<'r, DB> for Id<D>
    where
        DB: Database,
        D: IdDomain,
        i64: Decode<'r, DB>,
    {
        fn decode(value: <DB as Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
            let decoded = <i64 as Decode<'r, DB>>::decode(value)?;

            if decoded == 0 {
                return Err(Box::new(IdParseError::Zero));
            }

            if decoded < 0 {
                return Err(Box::new(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Negative database ID cannot be converted to domain-key::Id",
                )));
            }

            let as_u64 = u64::try_from(decoded).map_err(|_| {
                Box::new(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Negative database ID cannot be converted to domain-key::Id",
                )) as BoxDynError
            })?;

            Id::new(as_u64).ok_or_else(|| Box::new(IdParseError::Zero) as BoxDynError)
        }
    }

    // UUID uses backend-specific mappings:
    // - Postgres: native UUID type (`uuid`)
    // - SQLite/MySQL: string representation
    #[cfg(all(feature = "uuid", feature = "sqlx-postgres"))]
    impl<D> Type<sqlx::Postgres> for Uuid<D>
    where
        D: UuidDomain,
        ::uuid::Uuid: Type<sqlx::Postgres>,
    {
        fn type_info() -> <sqlx::Postgres as Database>::TypeInfo {
            <::uuid::Uuid as Type<sqlx::Postgres>>::type_info()
        }

        fn compatible(ty: &<sqlx::Postgres as Database>::TypeInfo) -> bool {
            <::uuid::Uuid as Type<sqlx::Postgres>>::compatible(ty)
        }
    }

    #[cfg(all(feature = "uuid", feature = "sqlx-postgres"))]
    impl<'q, D> Encode<'q, sqlx::Postgres> for Uuid<D>
    where
        D: UuidDomain,
        ::uuid::Uuid: Encode<'q, sqlx::Postgres>,
    {
        fn encode_by_ref(
            &self,
            buf: &mut <sqlx::Postgres as Database>::ArgumentBuffer<'q>,
        ) -> Result<IsNull, BoxDynError> {
            self.get().encode_by_ref(buf)
        }

        fn size_hint(&self) -> usize {
            core::mem::size_of::<u128>()
        }
    }

    #[cfg(all(feature = "uuid", feature = "sqlx-postgres"))]
    impl<'r, D> Decode<'r, sqlx::Postgres> for Uuid<D>
    where
        D: UuidDomain,
        ::uuid::Uuid: Decode<'r, sqlx::Postgres>,
    {
        fn decode(value: <sqlx::Postgres as Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
            let decoded = <::uuid::Uuid as Decode<'r, sqlx::Postgres>>::decode(value)?;
            Ok(Uuid::from(decoded))
        }
    }

    #[cfg(all(feature = "uuid", feature = "sqlx-sqlite"))]
    impl<D> Type<sqlx::Sqlite> for Uuid<D>
    where
        D: UuidDomain,
        String: Type<sqlx::Sqlite>,
    {
        fn type_info() -> <sqlx::Sqlite as Database>::TypeInfo {
            <String as Type<sqlx::Sqlite>>::type_info()
        }

        fn compatible(ty: &<sqlx::Sqlite as Database>::TypeInfo) -> bool {
            <String as Type<sqlx::Sqlite>>::compatible(ty)
        }
    }

    #[cfg(all(feature = "uuid", feature = "sqlx-sqlite"))]
    impl<'q, D> Encode<'q, sqlx::Sqlite> for Uuid<D>
    where
        D: UuidDomain,
        String: Encode<'q, sqlx::Sqlite>,
    {
        fn encode_by_ref(
            &self,
            buf: &mut <sqlx::Sqlite as Database>::ArgumentBuffer<'q>,
        ) -> Result<IsNull, BoxDynError> {
            <String as Encode<'q, sqlx::Sqlite>>::encode(self.to_string(), buf)
        }

        fn size_hint(&self) -> usize {
            36
        }
    }

    #[cfg(all(feature = "uuid", feature = "sqlx-sqlite"))]
    impl<'r, D> Decode<'r, sqlx::Sqlite> for Uuid<D>
    where
        D: UuidDomain,
        String: Decode<'r, sqlx::Sqlite>,
    {
        fn decode(value: <sqlx::Sqlite as Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
            let decoded = <String as Decode<'r, sqlx::Sqlite>>::decode(value)?;
            Uuid::parse(&decoded).map_err(|error| Box::new(error) as BoxDynError)
        }
    }

    #[cfg(all(feature = "uuid", feature = "sqlx-mysql"))]
    impl<D> Type<sqlx::MySql> for Uuid<D>
    where
        D: UuidDomain,
        String: Type<sqlx::MySql>,
    {
        fn type_info() -> <sqlx::MySql as Database>::TypeInfo {
            <String as Type<sqlx::MySql>>::type_info()
        }

        fn compatible(ty: &<sqlx::MySql as Database>::TypeInfo) -> bool {
            <String as Type<sqlx::MySql>>::compatible(ty)
        }
    }

    #[cfg(all(feature = "uuid", feature = "sqlx-mysql"))]
    impl<'q, D> Encode<'q, sqlx::MySql> for Uuid<D>
    where
        D: UuidDomain,
        String: Encode<'q, sqlx::MySql>,
    {
        fn encode_by_ref(
            &self,
            buf: &mut <sqlx::MySql as Database>::ArgumentBuffer<'q>,
        ) -> Result<IsNull, BoxDynError> {
            <String as Encode<'q, sqlx::MySql>>::encode(self.to_string(), buf)
        }

        fn size_hint(&self) -> usize {
            36
        }
    }

    #[cfg(all(feature = "uuid", feature = "sqlx-mysql"))]
    impl<'r, D> Decode<'r, sqlx::MySql> for Uuid<D>
    where
        D: UuidDomain,
        String: Decode<'r, sqlx::MySql>,
    {
        fn decode(value: <sqlx::MySql as Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
            let decoded = <String as Decode<'r, sqlx::MySql>>::decode(value)?;
            Uuid::parse(&decoded).map_err(|error| Box::new(error) as BoxDynError)
        }
    }

    // ULID uses backend-specific mappings:
    // - Postgres: native UUID type (`uuid`, 16 bytes)
    // - SQLite/MySQL: prefixed string representation
    #[cfg(all(feature = "ulid", feature = "sqlx-postgres"))]
    impl<D> Type<sqlx::Postgres> for Ulid<D>
    where
        D: UlidDomain,
        ::uuid::Uuid: Type<sqlx::Postgres>,
    {
        fn type_info() -> <sqlx::Postgres as Database>::TypeInfo {
            <::uuid::Uuid as Type<sqlx::Postgres>>::type_info()
        }

        fn compatible(ty: &<sqlx::Postgres as Database>::TypeInfo) -> bool {
            <::uuid::Uuid as Type<sqlx::Postgres>>::compatible(ty)
        }
    }

    #[cfg(all(feature = "ulid", feature = "sqlx-postgres"))]
    impl<'q, D> Encode<'q, sqlx::Postgres> for Ulid<D>
    where
        D: UlidDomain,
        ::uuid::Uuid: Encode<'q, sqlx::Postgres>,
    {
        fn encode_by_ref(
            &self,
            buf: &mut <sqlx::Postgres as Database>::ArgumentBuffer<'q>,
        ) -> Result<IsNull, BoxDynError> {
            let uuid = ::uuid::Uuid::from_bytes(self.to_bytes());
            uuid.encode_by_ref(buf)
        }

        fn size_hint(&self) -> usize {
            core::mem::size_of::<u128>()
        }
    }

    #[cfg(all(feature = "ulid", feature = "sqlx-postgres"))]
    impl<'r, D> Decode<'r, sqlx::Postgres> for Ulid<D>
    where
        D: UlidDomain,
        ::uuid::Uuid: Decode<'r, sqlx::Postgres>,
    {
        fn decode(value: <sqlx::Postgres as Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
            let decoded = <::uuid::Uuid as Decode<'r, sqlx::Postgres>>::decode(value)?;
            Ok(Ulid::from_bytes(*decoded.as_bytes()))
        }
    }

    #[cfg(all(feature = "ulid", feature = "sqlx-sqlite"))]
    impl<D> Type<sqlx::Sqlite> for Ulid<D>
    where
        D: UlidDomain,
        String: Type<sqlx::Sqlite>,
    {
        fn type_info() -> <sqlx::Sqlite as Database>::TypeInfo {
            <String as Type<sqlx::Sqlite>>::type_info()
        }

        fn compatible(ty: &<sqlx::Sqlite as Database>::TypeInfo) -> bool {
            <String as Type<sqlx::Sqlite>>::compatible(ty)
        }
    }

    #[cfg(all(feature = "ulid", feature = "sqlx-sqlite"))]
    impl<'q, D> Encode<'q, sqlx::Sqlite> for Ulid<D>
    where
        D: UlidDomain,
        String: Encode<'q, sqlx::Sqlite>,
    {
        fn encode_by_ref(
            &self,
            buf: &mut <sqlx::Sqlite as Database>::ArgumentBuffer<'q>,
        ) -> Result<IsNull, BoxDynError> {
            <String as Encode<'q, sqlx::Sqlite>>::encode(self.to_string(), buf)
        }

        fn size_hint(&self) -> usize {
            D::PREFIX.len() + 1 + 26
        }
    }

    #[cfg(all(feature = "ulid", feature = "sqlx-sqlite"))]
    impl<'r, D> Decode<'r, sqlx::Sqlite> for Ulid<D>
    where
        D: UlidDomain,
        String: Decode<'r, sqlx::Sqlite>,
    {
        fn decode(value: <sqlx::Sqlite as Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
            let decoded = <String as Decode<'r, sqlx::Sqlite>>::decode(value)?;
            Ulid::parse(&decoded).map_err(|error| Box::new(error) as BoxDynError)
        }
    }

    #[cfg(all(feature = "ulid", feature = "sqlx-mysql"))]
    impl<D> Type<sqlx::MySql> for Ulid<D>
    where
        D: UlidDomain,
        String: Type<sqlx::MySql>,
    {
        fn type_info() -> <sqlx::MySql as Database>::TypeInfo {
            <String as Type<sqlx::MySql>>::type_info()
        }

        fn compatible(ty: &<sqlx::MySql as Database>::TypeInfo) -> bool {
            <String as Type<sqlx::MySql>>::compatible(ty)
        }
    }

    #[cfg(all(feature = "ulid", feature = "sqlx-mysql"))]
    impl<'q, D> Encode<'q, sqlx::MySql> for Ulid<D>
    where
        D: UlidDomain,
        String: Encode<'q, sqlx::MySql>,
    {
        fn encode_by_ref(
            &self,
            buf: &mut <sqlx::MySql as Database>::ArgumentBuffer<'q>,
        ) -> Result<IsNull, BoxDynError> {
            <String as Encode<'q, sqlx::MySql>>::encode(self.to_string(), buf)
        }

        fn size_hint(&self) -> usize {
            D::PREFIX.len() + 1 + 26
        }
    }

    #[cfg(all(feature = "ulid", feature = "sqlx-mysql"))]
    impl<'r, D> Decode<'r, sqlx::MySql> for Ulid<D>
    where
        D: UlidDomain,
        String: Decode<'r, sqlx::MySql>,
    {
        fn decode(value: <sqlx::MySql as Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
            let decoded = <String as Decode<'r, sqlx::MySql>>::decode(value)?;
            Ulid::parse(&decoded).map_err(|error| Box::new(error) as BoxDynError)
        }
    }
}

#[cfg(feature = "axum")]
mod axum_support {
    use axum::http::StatusCode;
    use axum::response::{IntoResponse, Response};

    #[cfg(feature = "ulid")]
    use crate::UlidParseError;
    #[cfg(feature = "uuid")]
    use crate::UuidParseError;
    use crate::{IdParseError, KeyParseError};

    impl IntoResponse for KeyParseError {
        fn into_response(self) -> Response {
            (StatusCode::BAD_REQUEST, self.to_string()).into_response()
        }
    }

    impl IntoResponse for IdParseError {
        fn into_response(self) -> Response {
            (StatusCode::BAD_REQUEST, self.to_string()).into_response()
        }
    }

    #[cfg(feature = "uuid")]
    impl IntoResponse for UuidParseError {
        fn into_response(self) -> Response {
            (StatusCode::BAD_REQUEST, self.to_string()).into_response()
        }
    }

    #[cfg(feature = "ulid")]
    impl IntoResponse for UlidParseError {
        fn into_response(self) -> Response {
            (StatusCode::BAD_REQUEST, self.to_string()).into_response()
        }
    }
}

#[cfg(feature = "actix-web")]
mod actix_web_support {
    use actix_web::http::StatusCode;
    use actix_web::{HttpResponse, ResponseError};

    #[cfg(feature = "ulid")]
    use crate::UlidParseError;
    #[cfg(feature = "uuid")]
    use crate::UuidParseError;
    use crate::{IdParseError, KeyParseError};

    impl ResponseError for KeyParseError {
        fn status_code(&self) -> StatusCode {
            StatusCode::BAD_REQUEST
        }

        fn error_response(&self) -> HttpResponse {
            HttpResponse::build(self.status_code()).body(self.to_string())
        }
    }

    impl ResponseError for IdParseError {
        fn status_code(&self) -> StatusCode {
            StatusCode::BAD_REQUEST
        }

        fn error_response(&self) -> HttpResponse {
            HttpResponse::build(self.status_code()).body(self.to_string())
        }
    }

    #[cfg(feature = "uuid")]
    impl ResponseError for UuidParseError {
        fn status_code(&self) -> StatusCode {
            StatusCode::BAD_REQUEST
        }

        fn error_response(&self) -> HttpResponse {
            HttpResponse::build(self.status_code()).body(self.to_string())
        }
    }

    #[cfg(feature = "ulid")]
    impl ResponseError for UlidParseError {
        fn status_code(&self) -> StatusCode {
            StatusCode::BAD_REQUEST
        }

        fn error_response(&self) -> HttpResponse {
            HttpResponse::build(self.status_code()).body(self.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Domain, IdDomain, Key, KeyDomain};

    #[derive(Debug)]
    #[allow(dead_code)]
    struct TestKeyDomain;
    impl Domain for TestKeyDomain {
        const DOMAIN_NAME: &'static str = "test_key";
    }
    impl KeyDomain for TestKeyDomain {}
    #[allow(dead_code)]
    type TestKey = Key<TestKeyDomain>;

    #[derive(Debug)]
    #[allow(dead_code)]
    struct TestIdDomain;
    impl Domain for TestIdDomain {
        const DOMAIN_NAME: &'static str = "test_id";
    }
    impl IdDomain for TestIdDomain {}

    #[cfg(feature = "uuid")]
    #[derive(Debug)]
    struct TestUuidDomain;
    #[cfg(feature = "uuid")]
    impl Domain for TestUuidDomain {
        const DOMAIN_NAME: &'static str = "test_uuid";
    }
    #[cfg(feature = "uuid")]
    impl crate::UuidDomain for TestUuidDomain {}

    #[cfg(feature = "ulid")]
    #[derive(Debug)]
    struct TestUlidDomain;
    #[cfg(feature = "ulid")]
    impl Domain for TestUlidDomain {
        const DOMAIN_NAME: &'static str = "test_ulid";
    }
    #[cfg(feature = "ulid")]
    impl crate::UlidDomain for TestUlidDomain {
        const PREFIX: &'static str = "tst";
    }

    #[cfg(feature = "sqlx-postgres")]
    mod sqlx_postgres_traits {
        use super::*;
        use sqlx::decode::Decode;
        use sqlx::encode::Encode;
        use sqlx::{Postgres, Type};

        fn assert_sqlx_traits<T>()
        where
            T: Type<Postgres> + for<'q> Encode<'q, Postgres> + for<'r> Decode<'r, Postgres>,
        {
        }

        #[test]
        fn key_id_and_ulid_implement_sqlx_traits_for_postgres() {
            assert_sqlx_traits::<TestKey>();
            assert_sqlx_traits::<crate::Id<TestIdDomain>>();
            #[cfg(feature = "ulid")]
            assert_sqlx_traits::<crate::Ulid<TestUlidDomain>>();
        }

        #[test]
        fn key_and_id_are_compatible_with_expected_postgres_carriers() {
            assert!(<TestKey as Type<Postgres>>::compatible(&<String as Type<
                Postgres,
            >>::type_info(
            )));
            assert!(<crate::Id<TestIdDomain> as Type<Postgres>>::compatible(
                &<i64 as Type<Postgres>>::type_info()
            ));
        }

        #[cfg(feature = "uuid")]
        #[test]
        fn uuid_implements_sqlx_traits_for_postgres() {
            assert_sqlx_traits::<crate::Uuid<TestUuidDomain>>();
        }

        #[cfg(feature = "uuid")]
        #[test]
        fn uuid_is_compatible_with_native_postgres_uuid() {
            assert!(<crate::Uuid<TestUuidDomain> as Type<Postgres>>::compatible(
                &<::uuid::Uuid as Type<Postgres>>::type_info()
            ));
        }

        #[cfg(feature = "ulid")]
        #[test]
        fn ulid_postgres_uses_native_uuid_mapping() {
            assert!(<crate::Ulid<TestUlidDomain> as Type<Postgres>>::compatible(
                &<::uuid::Uuid as Type<Postgres>>::type_info()
            ));
        }

        #[cfg(feature = "ulid")]
        #[test]
        fn ulid_postgres_size_hint_is_binary_uuid_size() {
            let id = crate::Ulid::<TestUlidDomain>::new();
            let expected = core::mem::size_of::<u128>();
            let actual = <crate::Ulid<TestUlidDomain> as Encode<'_, Postgres>>::size_hint(&id);
            assert_eq!(actual, expected);
        }

        #[cfg(feature = "ulid")]
        #[test]
        fn ulid_to_bytes_matches_inner_and_uuid_conversion() {
            let id = crate::Ulid::<TestUlidDomain>::new();
            let outer_bytes = id.to_bytes();
            let inner_bytes = id.get().to_bytes();

            assert_eq!(outer_bytes, inner_bytes);

            let uuid = ::uuid::Uuid::from_bytes(outer_bytes);
            assert_eq!(*uuid.as_bytes(), outer_bytes);
        }
    }

    #[cfg(feature = "sqlx-sqlite")]
    mod sqlx_sqlite_traits {
        use super::*;
        use sqlx::decode::Decode;
        use sqlx::encode::Encode;
        use sqlx::sqlite::SqlitePoolOptions;
        use sqlx::{Sqlite, Type};

        fn assert_sqlx_traits<T>()
        where
            T: Type<Sqlite> + for<'q> Encode<'q, Sqlite> + for<'r> Decode<'r, Sqlite>,
        {
        }

        #[test]
        fn key_id_and_ulid_implement_sqlx_traits_for_sqlite() {
            assert_sqlx_traits::<TestKey>();
            assert_sqlx_traits::<crate::Id<TestIdDomain>>();
            #[cfg(feature = "ulid")]
            assert_sqlx_traits::<crate::Ulid<TestUlidDomain>>();
        }

        #[test]
        fn key_id_and_ulid_sqlite_compatibility_matches_carriers() {
            assert!(<TestKey as Type<Sqlite>>::compatible(&<String as Type<
                Sqlite,
            >>::type_info(
            )));
            assert!(<crate::Id<TestIdDomain> as Type<Sqlite>>::compatible(
                &<i64 as Type<Sqlite>>::type_info()
            ));
            #[cfg(feature = "ulid")]
            assert!(<crate::Ulid<TestUlidDomain> as Type<Sqlite>>::compatible(
                &<String as Type<Sqlite>>::type_info()
            ));
        }

        #[cfg(feature = "uuid")]
        #[test]
        fn uuid_implements_sqlx_traits_for_sqlite() {
            assert_sqlx_traits::<crate::Uuid<TestUuidDomain>>();
        }

        #[cfg(feature = "uuid")]
        #[test]
        fn uuid_sqlite_uses_string_compatibility() {
            assert!(<crate::Uuid<TestUuidDomain> as Type<Sqlite>>::compatible(
                &<String as Type<Sqlite>>::type_info()
            ));
            let value = crate::Uuid::<TestUuidDomain>::nil();
            let hint = <crate::Uuid<TestUuidDomain> as Encode<'_, Sqlite>>::size_hint(&value);
            assert_eq!(hint, 36);
        }

        #[tokio::test]
        async fn id_roundtrip_bind_and_query_scalar_sqlite() {
            let pool = SqlitePoolOptions::new()
                .max_connections(1)
                .connect("sqlite::memory:")
                .await
                .expect("create in-memory sqlite pool");

            sqlx::query("CREATE TABLE test_ids (id INTEGER NOT NULL PRIMARY KEY)")
                .execute(&pool)
                .await
                .expect("create test_ids table");

            let id = crate::Id::<TestIdDomain>::new(42).expect("non-zero id");

            sqlx::query("INSERT INTO test_ids (id) VALUES (?)")
                .bind(id)
                .execute(&pool)
                .await
                .expect("insert typed id");

            let row: crate::Id<TestIdDomain> = sqlx::query_scalar("SELECT id FROM test_ids")
                .fetch_one(&pool)
                .await
                .expect("fetch typed id");

            assert_eq!(id, row);
        }

        #[tokio::test]
        async fn key_roundtrip_bind_and_query_scalar_sqlite() {
            let pool = SqlitePoolOptions::new()
                .max_connections(1)
                .connect("sqlite::memory:")
                .await
                .expect("create in-memory sqlite pool");

            sqlx::query("CREATE TABLE test_keys (id TEXT NOT NULL PRIMARY KEY)")
                .execute(&pool)
                .await
                .expect("create test_keys table");

            let id = TestKey::new("exec_123").expect("valid key");

            sqlx::query("INSERT INTO test_keys (id) VALUES (?)")
                .bind(id.clone())
                .execute(&pool)
                .await
                .expect("insert typed key");

            let row: TestKey = sqlx::query_scalar("SELECT id FROM test_keys")
                .fetch_one(&pool)
                .await
                .expect("fetch typed key");

            assert_eq!(id, row);
        }

        #[cfg(feature = "uuid")]
        #[tokio::test]
        async fn uuid_roundtrip_bind_and_query_scalar_sqlite() {
            let pool = SqlitePoolOptions::new()
                .max_connections(1)
                .connect("sqlite::memory:")
                .await
                .expect("create in-memory sqlite pool");

            sqlx::query("CREATE TABLE test_uuids (id TEXT NOT NULL PRIMARY KEY)")
                .execute(&pool)
                .await
                .expect("create test_uuids table");

            let id = crate::Uuid::<TestUuidDomain>::parse("550e8400-e29b-41d4-a716-446655440000")
                .expect("valid uuid");

            sqlx::query("INSERT INTO test_uuids (id) VALUES (?)")
                .bind(id)
                .execute(&pool)
                .await
                .expect("insert typed uuid");

            let row: crate::Uuid<TestUuidDomain> = sqlx::query_scalar("SELECT id FROM test_uuids")
                .fetch_one(&pool)
                .await
                .expect("fetch typed uuid");

            assert_eq!(id, row);
        }

        #[cfg(feature = "ulid")]
        #[tokio::test]
        async fn ulid_roundtrip_bind_and_query_scalar_sqlite() {
            let pool = SqlitePoolOptions::new()
                .max_connections(1)
                .connect("sqlite::memory:")
                .await
                .expect("create in-memory sqlite pool");

            sqlx::query("CREATE TABLE test_ulids (id TEXT NOT NULL PRIMARY KEY)")
                .execute(&pool)
                .await
                .expect("create test_ulids table");

            let id = crate::Ulid::<TestUlidDomain>::new();

            sqlx::query("INSERT INTO test_ulids (id) VALUES (?)")
                .bind(id)
                .execute(&pool)
                .await
                .expect("insert typed ulid");

            let row: crate::Ulid<TestUlidDomain> = sqlx::query_scalar("SELECT id FROM test_ulids")
                .fetch_one(&pool)
                .await
                .expect("fetch typed ulid");

            assert_eq!(id, row);
        }

        #[cfg(feature = "ulid")]
        #[tokio::test]
        async fn ulid_sqlite_decode_rejects_wrong_prefix() {
            let pool = SqlitePoolOptions::new()
                .max_connections(1)
                .connect("sqlite::memory:")
                .await
                .expect("create in-memory sqlite pool");

            sqlx::query("CREATE TABLE bad_ulids (id TEXT NOT NULL PRIMARY KEY)")
                .execute(&pool)
                .await
                .expect("create bad_ulids table");

            sqlx::query("INSERT INTO bad_ulids (id) VALUES (?)")
                .bind("bad_01D39ZY06FGSCTVN4T2V9PKHFZ")
                .execute(&pool)
                .await
                .expect("insert invalid-prefix ulid");

            let result: Result<crate::Ulid<TestUlidDomain>, sqlx::Error> =
                sqlx::query_scalar("SELECT id FROM bad_ulids")
                    .fetch_one(&pool)
                    .await;

            assert!(result.is_err(), "decode should fail for wrong ULID prefix");
        }
    }

    #[cfg(feature = "sqlx-mysql")]
    mod sqlx_mysql_traits {
        use super::*;
        use sqlx::decode::Decode;
        use sqlx::encode::Encode;
        use sqlx::{MySql, Type};

        fn assert_sqlx_traits<T>()
        where
            T: Type<MySql> + for<'q> Encode<'q, MySql> + for<'r> Decode<'r, MySql>,
        {
        }

        #[test]
        fn key_id_and_ulid_implement_sqlx_traits_for_mysql() {
            assert_sqlx_traits::<TestKey>();
            assert_sqlx_traits::<crate::Id<TestIdDomain>>();
            #[cfg(feature = "ulid")]
            assert_sqlx_traits::<crate::Ulid<TestUlidDomain>>();
        }

        #[test]
        fn key_id_and_ulid_mysql_compatibility_matches_carriers() {
            assert!(<TestKey as Type<MySql>>::compatible(&<String as Type<
                MySql,
            >>::type_info(
            )));
            assert!(<crate::Id<TestIdDomain> as Type<MySql>>::compatible(
                &<i64 as Type<MySql>>::type_info()
            ));
            #[cfg(feature = "ulid")]
            assert!(<crate::Ulid<TestUlidDomain> as Type<MySql>>::compatible(
                &<String as Type<MySql>>::type_info()
            ));
        }

        #[cfg(feature = "uuid")]
        #[test]
        fn uuid_implements_sqlx_traits_for_mysql() {
            assert_sqlx_traits::<crate::Uuid<TestUuidDomain>>();
        }

        #[cfg(feature = "uuid")]
        #[test]
        fn uuid_mysql_uses_string_compatibility() {
            assert!(<crate::Uuid<TestUuidDomain> as Type<MySql>>::compatible(
                &<String as Type<MySql>>::type_info()
            ));
            let value = crate::Uuid::<TestUuidDomain>::nil();
            let hint = <crate::Uuid<TestUuidDomain> as Encode<'_, MySql>>::size_hint(&value);
            assert_eq!(hint, 36);
        }
    }

    #[cfg(all(feature = "axum", feature = "serde"))]
    mod axum_responses {
        use super::*;
        use crate::{IdParseError, KeyParseError};
        use axum::body::{to_bytes, Body};
        use axum::extract::{Form, Json};
        use axum::http::{header, Request, StatusCode};
        use axum::response::IntoResponse;
        use axum::routing::get;
        use axum::Router;
        use serde::Deserialize;
        use tower::util::ServiceExt;

        #[derive(Debug, Deserialize)]
        struct KeyPayload {
            id: TestKey,
            title: String,
        }

        #[derive(Debug, Deserialize)]
        struct IdPayload {
            id: crate::Id<TestIdDomain>,
            title: String,
        }

        #[cfg(feature = "uuid")]
        #[derive(Debug, Deserialize)]
        struct UuidPayload {
            id: crate::Uuid<TestUuidDomain>,
            title: String,
        }

        #[cfg(feature = "ulid")]
        #[derive(Debug, Deserialize)]
        struct UlidPayload {
            id: crate::Ulid<TestUlidDomain>,
            title: String,
        }

        #[test]
        fn key_error_maps_to_bad_request() {
            let response = KeyParseError::Empty.into_response();
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        }

        #[test]
        fn id_error_maps_to_bad_request() {
            let response = IdParseError::Zero.into_response();
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        }

        #[cfg(feature = "uuid")]
        #[test]
        fn uuid_error_maps_to_bad_request() {
            let err = crate::Uuid::<TestUuidDomain>::parse("not-a-uuid").unwrap_err();
            let response = err.into_response();
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        }

        #[cfg(feature = "ulid")]
        #[test]
        fn ulid_error_maps_to_bad_request() {
            let err = crate::Ulid::<TestUlidDomain>::parse("bad").unwrap_err();
            let response = err.into_response();
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        }

        #[tokio::test]
        async fn e2e_handler_error_returns_bad_request_for_key() {
            async fn handler() -> Result<&'static str, KeyParseError> {
                Err(KeyParseError::Empty)
            }

            let app = Router::new().route("/key", get(handler));
            let response = app
                .oneshot(
                    Request::builder()
                        .uri("/key")
                        .method("GET")
                        .body(Body::empty())
                        .expect("build request"),
                )
                .await
                .expect("execute request");

            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
            let body = to_bytes(response.into_body(), usize::MAX)
                .await
                .expect("read response body");
            let text = String::from_utf8(body.to_vec()).expect("utf8 body");
            assert!(text.contains("Key cannot be empty"));
        }

        #[tokio::test]
        async fn e2e_handler_error_returns_bad_request_for_id() {
            async fn handler() -> Result<&'static str, IdParseError> {
                Err(IdParseError::Zero)
            }

            let app = Router::new().route("/id", get(handler));
            let response = app
                .oneshot(
                    Request::builder()
                        .uri("/id")
                        .method("GET")
                        .body(Body::empty())
                        .expect("build request"),
                )
                .await
                .expect("execute request");

            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
            let body = to_bytes(response.into_body(), usize::MAX)
                .await
                .expect("read response body");
            let text = String::from_utf8(body.to_vec()).expect("utf8 body");
            assert!(text.contains("ID cannot be zero"));
        }

        #[cfg(feature = "uuid")]
        #[tokio::test]
        async fn e2e_handler_error_returns_bad_request_for_uuid() {
            async fn handler() -> Result<&'static str, crate::UuidParseError> {
                crate::Uuid::<TestUuidDomain>::parse("not-a-uuid")?;
                Ok("ok")
            }

            let app = Router::new().route("/uuid", get(handler));
            let response = app
                .oneshot(
                    Request::builder()
                        .uri("/uuid")
                        .method("GET")
                        .body(Body::empty())
                        .expect("build request"),
                )
                .await
                .expect("execute request");

            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        }

        #[cfg(feature = "ulid")]
        #[tokio::test]
        async fn e2e_handler_error_returns_bad_request_for_ulid() {
            async fn handler() -> Result<&'static str, crate::UlidParseError> {
                crate::Ulid::<TestUlidDomain>::parse("bad")?;
                Ok("ok")
            }

            let app = Router::new().route("/ulid", get(handler));
            let response = app
                .oneshot(
                    Request::builder()
                        .uri("/ulid")
                        .method("GET")
                        .body(Body::empty())
                        .expect("build request"),
                )
                .await
                .expect("execute request");

            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        }

        #[tokio::test]
        async fn e2e_json_payload_roundtrip_for_key_and_id() {
            async fn key_handler(Json(payload): Json<KeyPayload>) -> &'static str {
                assert_eq!(payload.id.as_str(), "valid_key");
                assert_eq!(payload.title, "hello");
                "ok"
            }
            async fn id_handler(Json(payload): Json<IdPayload>) -> &'static str {
                assert_eq!(payload.id.get(), 42);
                assert_eq!(payload.title, "hello");
                "ok"
            }

            let app = Router::new()
                .route("/json-key", axum::routing::post(key_handler))
                .route("/json-id", axum::routing::post(id_handler));

            let key_ok = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri("/json-key")
                        .method("POST")
                        .header(header::CONTENT_TYPE, "application/json")
                        .body(Body::from(r#"{"id":"valid_key","title":"hello"}"#))
                        .expect("build key request"),
                )
                .await
                .expect("execute key request");
            assert_eq!(key_ok.status(), StatusCode::OK);

            let id_ok = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri("/json-id")
                        .method("POST")
                        .header(header::CONTENT_TYPE, "application/json")
                        .body(Body::from(r#"{"id":42,"title":"hello"}"#))
                        .expect("build id request"),
                )
                .await
                .expect("execute id request");
            assert_eq!(id_ok.status(), StatusCode::OK);

            let key_bad = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri("/json-key")
                        .method("POST")
                        .header(header::CONTENT_TYPE, "application/json")
                        .body(Body::from(r#"{"id":"bad key","title":"hello"}"#))
                        .expect("build bad key request"),
                )
                .await
                .expect("execute bad key request");
            assert!(key_bad.status().is_client_error());

            let id_bad = app
                .oneshot(
                    Request::builder()
                        .uri("/json-id")
                        .method("POST")
                        .header(header::CONTENT_TYPE, "application/json")
                        .body(Body::from(r#"{"id":0,"title":"hello"}"#))
                        .expect("build bad id request"),
                )
                .await
                .expect("execute bad id request");
            assert!(id_bad.status().is_client_error());
        }

        #[tokio::test]
        async fn e2e_form_payload_roundtrip_for_key_and_id() {
            async fn key_handler(Form(payload): Form<KeyPayload>) -> &'static str {
                assert_eq!(payload.id.as_str(), "valid_key");
                assert_eq!(payload.title, "hello");
                "ok"
            }
            async fn id_handler(Form(payload): Form<IdPayload>) -> &'static str {
                assert_eq!(payload.id.get(), 42);
                assert_eq!(payload.title, "hello");
                "ok"
            }

            let app = Router::new()
                .route("/form-key", axum::routing::post(key_handler))
                .route("/form-id", axum::routing::post(id_handler));

            let key_ok = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri("/form-key")
                        .method("POST")
                        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                        .body(Body::from("id=valid_key&title=hello"))
                        .expect("build key form request"),
                )
                .await
                .expect("execute key form request");
            assert_eq!(key_ok.status(), StatusCode::OK);

            let id_ok = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri("/form-id")
                        .method("POST")
                        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                        .body(Body::from("id=42&title=hello"))
                        .expect("build id form request"),
                )
                .await
                .expect("execute id form request");
            assert_eq!(id_ok.status(), StatusCode::OK);

            let key_bad = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri("/form-key")
                        .method("POST")
                        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                        .body(Body::from("id=bad%20key&title=hello"))
                        .expect("build bad key form request"),
                )
                .await
                .expect("execute bad key form request");
            assert!(key_bad.status().is_client_error());

            let id_bad = app
                .oneshot(
                    Request::builder()
                        .uri("/form-id")
                        .method("POST")
                        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                        .body(Body::from("id=0&title=hello"))
                        .expect("build bad id form request"),
                )
                .await
                .expect("execute bad id form request");
            assert!(id_bad.status().is_client_error());
        }

        #[cfg(feature = "uuid")]
        #[tokio::test]
        async fn e2e_json_and_form_payload_roundtrip_for_uuid() {
            async fn json_handler(Json(payload): Json<UuidPayload>) -> &'static str {
                assert_eq!(
                    payload.id.to_string(),
                    "550e8400-e29b-41d4-a716-446655440000"
                );
                assert_eq!(payload.title, "hello");
                "ok"
            }
            async fn form_handler(Form(payload): Form<UuidPayload>) -> &'static str {
                assert_eq!(
                    payload.id.to_string(),
                    "550e8400-e29b-41d4-a716-446655440000"
                );
                assert_eq!(payload.title, "hello");
                "ok"
            }

            let app = Router::new()
                .route("/json-uuid", axum::routing::post(json_handler))
                .route("/form-uuid", axum::routing::post(form_handler));

            let json_ok = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri("/json-uuid")
                        .method("POST")
                        .header(header::CONTENT_TYPE, "application/json")
                        .body(Body::from(
                            r#"{"id":"550e8400-e29b-41d4-a716-446655440000","title":"hello"}"#,
                        ))
                        .expect("build uuid json request"),
                )
                .await
                .expect("execute uuid json request");
            assert_eq!(json_ok.status(), StatusCode::OK);

            let form_ok = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri("/form-uuid")
                        .method("POST")
                        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                        .body(Body::from(
                            "id=550e8400-e29b-41d4-a716-446655440000&title=hello",
                        ))
                        .expect("build uuid form request"),
                )
                .await
                .expect("execute uuid form request");
            assert_eq!(form_ok.status(), StatusCode::OK);

            let json_bad = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri("/json-uuid")
                        .method("POST")
                        .header(header::CONTENT_TYPE, "application/json")
                        .body(Body::from(r#"{"id":"not-a-uuid","title":"hello"}"#))
                        .expect("build bad uuid json request"),
                )
                .await
                .expect("execute bad uuid json request");
            assert!(json_bad.status().is_client_error());

            let form_bad = app
                .oneshot(
                    Request::builder()
                        .uri("/form-uuid")
                        .method("POST")
                        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                        .body(Body::from("id=not-a-uuid&title=hello"))
                        .expect("build bad uuid form request"),
                )
                .await
                .expect("execute bad uuid form request");
            assert!(form_bad.status().is_client_error());
        }

        #[cfg(feature = "ulid")]
        #[tokio::test]
        async fn e2e_json_and_form_payload_roundtrip_for_ulid() {
            async fn json_handler(Json(payload): Json<UlidPayload>) -> &'static str {
                assert!(payload.id.to_string().starts_with("tst_"));
                assert_eq!(payload.title, "hello");
                "ok"
            }
            async fn form_handler(Form(payload): Form<UlidPayload>) -> &'static str {
                assert!(payload.id.to_string().starts_with("tst_"));
                assert_eq!(payload.title, "hello");
                "ok"
            }

            let app = Router::new()
                .route("/json-ulid", axum::routing::post(json_handler))
                .route("/form-ulid", axum::routing::post(form_handler));

            let json_ok = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri("/json-ulid")
                        .method("POST")
                        .header(header::CONTENT_TYPE, "application/json")
                        .body(Body::from(
                            r#"{"id":"tst_01D39ZY06FGSCTVN4T2V9PKHFZ","title":"hello"}"#,
                        ))
                        .expect("build ulid json request"),
                )
                .await
                .expect("execute ulid json request");
            assert_eq!(json_ok.status(), StatusCode::OK);

            let form_ok = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri("/form-ulid")
                        .method("POST")
                        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                        .body(Body::from("id=tst_01D39ZY06FGSCTVN4T2V9PKHFZ&title=hello"))
                        .expect("build ulid form request"),
                )
                .await
                .expect("execute ulid form request");
            assert_eq!(form_ok.status(), StatusCode::OK);

            let json_bad = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri("/json-ulid")
                        .method("POST")
                        .header(header::CONTENT_TYPE, "application/json")
                        .body(Body::from(r#"{"id":"bad","title":"hello"}"#))
                        .expect("build bad ulid json request"),
                )
                .await
                .expect("execute bad ulid json request");
            assert!(json_bad.status().is_client_error());

            let form_bad = app
                .oneshot(
                    Request::builder()
                        .uri("/form-ulid")
                        .method("POST")
                        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                        .body(Body::from("id=bad&title=hello"))
                        .expect("build bad ulid form request"),
                )
                .await
                .expect("execute bad ulid form request");
            assert!(form_bad.status().is_client_error());
        }
    }

    #[cfg(all(feature = "actix-web", feature = "serde"))]
    mod actix_responses {
        use super::*;
        use crate::{IdParseError, KeyParseError};
        use actix_web::http::StatusCode;
        use actix_web::{test, web, App, HttpResponse, ResponseError};
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Deserialize, Serialize)]
        struct KeyPayload {
            id: TestKey,
            title: String,
        }

        #[derive(Debug, Deserialize, Serialize)]
        struct IdPayload {
            id: crate::Id<TestIdDomain>,
            title: String,
        }

        #[cfg(feature = "uuid")]
        #[derive(Debug, Deserialize, Serialize)]
        struct UuidPayload {
            id: crate::Uuid<TestUuidDomain>,
            title: String,
        }

        #[cfg(feature = "ulid")]
        #[derive(Debug, Deserialize, Serialize)]
        struct UlidPayload {
            id: crate::Ulid<TestUlidDomain>,
            title: String,
        }

        #[test]
        fn key_error_maps_to_bad_request() {
            assert_eq!(KeyParseError::Empty.status_code(), StatusCode::BAD_REQUEST);
        }

        #[test]
        fn id_error_maps_to_bad_request() {
            assert_eq!(IdParseError::Zero.status_code(), StatusCode::BAD_REQUEST);
        }

        #[cfg(feature = "uuid")]
        #[test]
        fn uuid_error_maps_to_bad_request() {
            let err = crate::Uuid::<TestUuidDomain>::parse("not-a-uuid").unwrap_err();
            assert_eq!(err.status_code(), StatusCode::BAD_REQUEST);
        }

        #[cfg(feature = "ulid")]
        #[test]
        fn ulid_error_maps_to_bad_request() {
            let err = crate::Ulid::<TestUlidDomain>::parse("bad").unwrap_err();
            assert_eq!(err.status_code(), StatusCode::BAD_REQUEST);
        }

        #[tokio::test]
        async fn e2e_handler_error_returns_bad_request_for_key() {
            async fn handler() -> Result<HttpResponse, KeyParseError> {
                Err(KeyParseError::Empty)
            }

            let app = test::init_service(App::new().route("/key", web::get().to(handler))).await;
            let req = test::TestRequest::get().uri("/key").to_request();
            let resp = test::call_service(&app, req).await;

            assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
            let body = test::read_body(resp).await;
            let text = String::from_utf8(body.to_vec()).expect("utf8 body");
            assert!(text.contains("Key cannot be empty"));
        }

        #[tokio::test]
        async fn e2e_handler_error_returns_bad_request_for_id() {
            async fn handler() -> Result<HttpResponse, IdParseError> {
                Err(IdParseError::Zero)
            }

            let app = test::init_service(App::new().route("/id", web::get().to(handler))).await;
            let req = test::TestRequest::get().uri("/id").to_request();
            let resp = test::call_service(&app, req).await;

            assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
            let body = test::read_body(resp).await;
            let text = String::from_utf8(body.to_vec()).expect("utf8 body");
            assert!(text.contains("ID cannot be zero"));
        }

        #[cfg(feature = "uuid")]
        #[tokio::test]
        async fn e2e_handler_error_returns_bad_request_for_uuid() {
            async fn handler() -> Result<HttpResponse, crate::UuidParseError> {
                crate::Uuid::<TestUuidDomain>::parse("not-a-uuid")?;
                Ok(HttpResponse::Ok().finish())
            }

            let app = test::init_service(App::new().route("/uuid", web::get().to(handler))).await;
            let req = test::TestRequest::get().uri("/uuid").to_request();
            let resp = test::call_service(&app, req).await;

            assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        }

        #[cfg(feature = "ulid")]
        #[tokio::test]
        async fn e2e_handler_error_returns_bad_request_for_ulid() {
            async fn handler() -> Result<HttpResponse, crate::UlidParseError> {
                crate::Ulid::<TestUlidDomain>::parse("bad")?;
                Ok(HttpResponse::Ok().finish())
            }

            let app = test::init_service(App::new().route("/ulid", web::get().to(handler))).await;
            let req = test::TestRequest::get().uri("/ulid").to_request();
            let resp = test::call_service(&app, req).await;

            assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        }

        #[tokio::test]
        async fn e2e_json_payload_roundtrip_for_key_and_id() {
            async fn key_handler(payload: web::Json<KeyPayload>) -> HttpResponse {
                assert_eq!(payload.id.as_str(), "valid_key");
                assert_eq!(payload.title, "hello");
                HttpResponse::Ok().finish()
            }
            async fn id_handler(payload: web::Json<IdPayload>) -> HttpResponse {
                assert_eq!(payload.id.get(), 42);
                assert_eq!(payload.title, "hello");
                HttpResponse::Ok().finish()
            }

            let app = test::init_service(
                App::new()
                    .route("/json-key", web::post().to(key_handler))
                    .route("/json-id", web::post().to(id_handler)),
            )
            .await;

            let key_ok = test::TestRequest::post()
                .uri("/json-key")
                .set_json(&serde_json::json!({"id": "valid_key", "title": "hello"}))
                .to_request();
            let key_resp = test::call_service(&app, key_ok).await;
            assert_eq!(key_resp.status(), StatusCode::OK);

            let id_ok = test::TestRequest::post()
                .uri("/json-id")
                .set_json(&serde_json::json!({"id": 42, "title": "hello"}))
                .to_request();
            let id_resp = test::call_service(&app, id_ok).await;
            assert_eq!(id_resp.status(), StatusCode::OK);

            let key_bad = test::TestRequest::post()
                .uri("/json-key")
                .set_json(&serde_json::json!({"id": "bad key", "title": "hello"}))
                .to_request();
            let key_bad_resp = test::call_service(&app, key_bad).await;
            assert!(key_bad_resp.status().is_client_error());

            let id_bad = test::TestRequest::post()
                .uri("/json-id")
                .set_json(&serde_json::json!({"id": 0, "title": "hello"}))
                .to_request();
            let id_bad_resp = test::call_service(&app, id_bad).await;
            assert!(id_bad_resp.status().is_client_error());
        }

        #[tokio::test]
        async fn e2e_form_payload_roundtrip_for_key_and_id() {
            async fn key_handler(payload: web::Form<KeyPayload>) -> HttpResponse {
                assert_eq!(payload.id.as_str(), "valid_key");
                assert_eq!(payload.title, "hello");
                HttpResponse::Ok().finish()
            }
            async fn id_handler(payload: web::Form<IdPayload>) -> HttpResponse {
                assert_eq!(payload.id.get(), 42);
                assert_eq!(payload.title, "hello");
                HttpResponse::Ok().finish()
            }

            let app = test::init_service(
                App::new()
                    .route("/form-key", web::post().to(key_handler))
                    .route("/form-id", web::post().to(id_handler)),
            )
            .await;

            let key_ok = test::TestRequest::post()
                .uri("/form-key")
                .set_form(&KeyPayload {
                    id: TestKey::new("valid_key").expect("valid key"),
                    title: "hello".to_string(),
                })
                .to_request();
            let key_resp = test::call_service(&app, key_ok).await;
            assert_eq!(key_resp.status(), StatusCode::OK);

            let id_ok = test::TestRequest::post()
                .uri("/form-id")
                .set_form(&IdPayload {
                    id: crate::Id::<TestIdDomain>::new(42).expect("non-zero id"),
                    title: "hello".to_string(),
                })
                .to_request();
            let id_resp = test::call_service(&app, id_ok).await;
            assert_eq!(id_resp.status(), StatusCode::OK);

            let key_bad = test::TestRequest::post()
                .uri("/form-key")
                .insert_header(("content-type", "application/x-www-form-urlencoded"))
                .set_payload("id=bad%20key&title=hello")
                .to_request();
            let key_bad_resp = test::call_service(&app, key_bad).await;
            assert!(key_bad_resp.status().is_client_error());

            let id_bad = test::TestRequest::post()
                .uri("/form-id")
                .insert_header(("content-type", "application/x-www-form-urlencoded"))
                .set_payload("id=0&title=hello")
                .to_request();
            let id_bad_resp = test::call_service(&app, id_bad).await;
            assert!(id_bad_resp.status().is_client_error());
        }

        #[cfg(feature = "uuid")]
        #[tokio::test]
        async fn e2e_json_and_form_payload_roundtrip_for_uuid() {
            async fn json_handler(payload: web::Json<UuidPayload>) -> HttpResponse {
                assert_eq!(
                    payload.id.to_string(),
                    "550e8400-e29b-41d4-a716-446655440000"
                );
                assert_eq!(payload.title, "hello");
                HttpResponse::Ok().finish()
            }
            async fn form_handler(payload: web::Form<UuidPayload>) -> HttpResponse {
                assert_eq!(
                    payload.id.to_string(),
                    "550e8400-e29b-41d4-a716-446655440000"
                );
                assert_eq!(payload.title, "hello");
                HttpResponse::Ok().finish()
            }

            let app = test::init_service(
                App::new()
                    .route("/json-uuid", web::post().to(json_handler))
                    .route("/form-uuid", web::post().to(form_handler)),
            )
            .await;

            let json_ok = test::TestRequest::post()
                .uri("/json-uuid")
                .set_json(
                    &serde_json::json!({"id":"550e8400-e29b-41d4-a716-446655440000","title":"hello"}),
                )
                .to_request();
            let json_resp = test::call_service(&app, json_ok).await;
            assert_eq!(json_resp.status(), StatusCode::OK);

            let form_ok = test::TestRequest::post()
                .uri("/form-uuid")
                .set_form(&UuidPayload {
                    id: crate::Uuid::<TestUuidDomain>::parse(
                        "550e8400-e29b-41d4-a716-446655440000",
                    )
                    .expect("valid uuid"),
                    title: "hello".to_string(),
                })
                .to_request();
            let form_resp = test::call_service(&app, form_ok).await;
            assert_eq!(form_resp.status(), StatusCode::OK);

            let json_bad = test::TestRequest::post()
                .uri("/json-uuid")
                .set_json(&serde_json::json!({"id":"not-a-uuid","title":"hello"}))
                .to_request();
            let json_bad_resp = test::call_service(&app, json_bad).await;
            assert!(json_bad_resp.status().is_client_error());

            let form_bad = test::TestRequest::post()
                .uri("/form-uuid")
                .insert_header(("content-type", "application/x-www-form-urlencoded"))
                .set_payload("id=not-a-uuid&title=hello")
                .to_request();
            let form_bad_resp = test::call_service(&app, form_bad).await;
            assert!(form_bad_resp.status().is_client_error());
        }

        #[cfg(feature = "ulid")]
        #[tokio::test]
        async fn e2e_json_and_form_payload_roundtrip_for_ulid() {
            async fn json_handler(payload: web::Json<UlidPayload>) -> HttpResponse {
                assert!(payload.id.to_string().starts_with("tst_"));
                assert_eq!(payload.title, "hello");
                HttpResponse::Ok().finish()
            }
            async fn form_handler(payload: web::Form<UlidPayload>) -> HttpResponse {
                assert!(payload.id.to_string().starts_with("tst_"));
                assert_eq!(payload.title, "hello");
                HttpResponse::Ok().finish()
            }

            let app = test::init_service(
                App::new()
                    .route("/json-ulid", web::post().to(json_handler))
                    .route("/form-ulid", web::post().to(form_handler)),
            )
            .await;

            let json_ok = test::TestRequest::post()
                .uri("/json-ulid")
                .set_json(
                    &serde_json::json!({"id":"tst_01D39ZY06FGSCTVN4T2V9PKHFZ","title":"hello"}),
                )
                .to_request();
            let json_resp = test::call_service(&app, json_ok).await;
            assert_eq!(json_resp.status(), StatusCode::OK);

            let form_ok = test::TestRequest::post()
                .uri("/form-ulid")
                .set_form(&UlidPayload {
                    id: crate::Ulid::<TestUlidDomain>::parse("tst_01D39ZY06FGSCTVN4T2V9PKHFZ")
                        .expect("valid ulid"),
                    title: "hello".to_string(),
                })
                .to_request();
            let form_resp = test::call_service(&app, form_ok).await;
            assert_eq!(form_resp.status(), StatusCode::OK);

            let json_bad = test::TestRequest::post()
                .uri("/json-ulid")
                .set_json(&serde_json::json!({"id":"bad","title":"hello"}))
                .to_request();
            let json_bad_resp = test::call_service(&app, json_bad).await;
            assert!(json_bad_resp.status().is_client_error());

            let form_bad = test::TestRequest::post()
                .uri("/form-ulid")
                .insert_header(("content-type", "application/x-www-form-urlencoded"))
                .set_payload("id=bad&title=hello")
                .to_request();
            let form_bad_resp = test::call_service(&app, form_bad).await;
            assert!(form_bad_resp.status().is_client_error());
        }
    }
}
