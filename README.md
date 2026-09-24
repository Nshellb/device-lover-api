# device-lover-api

Rust/Axum + PostgreSQL 기반 API. health, users, auth와 공개 기기 카탈로그 조회 API를 제공한다.

## Docker로 실행

Docker Desktop을 실행한 뒤 이 디렉터리에서 실행한다.

```bash
docker compose up --build
```

API는 `http://localhost:4040`에서 실행된다. 앱 시작 시 PostgreSQL 연결과 SQLx migration이 자동으로 수행된다. 종료는 `Ctrl-C`, 백그라운드 컨테이너 정리는 다음 명령으로 한다.

```bash
docker compose down
```

개발 DB까지 초기화해야 할 때만 다음 명령을 사용한다. 기존 사용자와 카탈로그 데이터가 삭제된다.

```bash
docker compose down -v
```

## Rust를 직접 실행

Rust가 없다면 먼저 [rustup](https://rustup.rs)을 설치하고 새 터미널을 열거나 현재 셸에 다음을 적용한다.

```bash
source "$HOME/.cargo/env"
```

PostgreSQL 16을 먼저 실행하고 `device_lover` 데이터베이스와 사용자를 만든다. 기본 설정은 다음 환경변수와 일치해야 한다.

```bash
export DATABASE_URL='postgres://device_lover:device_lover@127.0.0.1:5432/device_lover'
export JWT_SECRET='local-dev-secret-change-me'
export HOST='127.0.0.1'
export PORT='4040'
cargo run
```

앱이 시작할 때 `migrations/`의 모든 migration을 자동으로 실행한다. PostgreSQL을 Homebrew로 설치한 경우 예시는 다음과 같다.

```bash
brew install postgresql@16
brew services start postgresql@16
createuser device_lover
psql -d postgres -c "ALTER USER device_lover WITH PASSWORD 'device_lover';"
createdb -O device_lover device_lover
cargo run
```

`device_lover` 사용자가 이미 있으면 `createuser`는 건너뛴다. 로컬 PostgreSQL의 인증 설정에 따라 `DATABASE_URL`의 사용자·비밀번호를 맞춘다.

## 확인

```bash
curl http://localhost:4040/health
curl http://localhost:4040/api/v1/catalog/schema
curl 'http://localhost:4040/api/v1/devices?page_size=20'
open http://localhost:4040/swagger-ui
```

스마트폰 카탈로그에는 초기 제품 seed가 없다. 카메라는 `0005_create_cameras.sql`에서 별도 `camera_models` 테이블을 만들고, `0006` migration에서 캐논 EOS 5D 시리즈 6종(5DS / 5DS R 포함), 6D 시리즈 2종, 10D~90D 9종을 입력한다.

BO와 FO 카메라 목록은 각각 `GET /api/v1/admin/cameras`, `GET /api/v1/cameras`에서 조회한다. `q`(최대 100자, 모델명·Canon·캐논 검색), `series`(`EOS 5D`, `EOS 6D`, `EOS x0D`), `page`(기본 1, 최대 10000), `page_size`(기본 30, 최대 100)를 지원한다. 카메라끼리의 비교는 `GET /api/v1/cameras/comparisons`를 사용한다. 스마트폰은 `device_models`, 카메라는 `camera_models`에 저장하며 두 카테고리를 한 비교 응답에 섞지 않는다.

카메라 정보는 브랜드 FK, 모델명·시리즈, 출시 월(`releaseMonth`, `YYYY-MM`), 센서 형식·유효 화소·이미지 프로세서·렌즈 마운트·최대 연사와 조건·동영상 사양·본체 무게를 저장한다. 출시 월은 Canon Camera Museum의 출시 월이며 한국 출시일을 뜻하지 않는다. 무게는 배터리·메모리 카드·렌즈를 제외한 본체 기준이다. 출처 URL·제목·확인 시각을 함께 보관하며, BO에서 공식 출처 링크를 확인할 수 있다. [캐논 카메라 데이터와 출처](docs/canon-cameras.md)에 모델별 값과 기준을 정리했다.

FO의 잘못된 기기 경로는 `POST /api/v1/route-misses`로 수집해 `route_miss_events`에 저장한다. 요청 경로, 쿼리·프래그먼트를 제거한 유입 경로, 브라우저 언어, 화면 크기 구간과 발생 시각만 기록하며 IP나 사용자 식별자는 저장하지 않는다. `GET /api/v1/admin/route-misses?days=30&limit=50`에서 경로별 발생 횟수와 최초·최근 발생 시각을 조회할 수 있다.

검색 결과의 `기기 보기`와 `비교에 추가` 선택은 `POST /api/v1/device-selections`에서 일별 누적으로 기록한다. `GET /api/v1/popular-devices`는 최근 30일 선택 횟수를 기준으로 스마트폰 5개와 카메라 5개를 반환하며, 데이터가 부족한 자리는 카테고리별 최신 기기로 채운다. 이 응답은 API 프로세스 메모리에 5분간 캐싱한다. 검색어와 IP, 사용자 식별자는 저장하지 않는다.

- [설계 문서 · ERD · 프론트 매핑](docs/catalog-design.md)
- [PostgreSQL DDL 초안](docs/catalog-schema.sql)
- [OpenAPI 3.1 계약 초안](docs/catalog-openapi.yaml)

현재 구현된 계약은 `/api-docs/openapi.json`, Swagger UI는 `/swagger-ui`에서 확인한다. 카탈로그 설계와 seed 준비 사항은 [설계 문서](docs/catalog-design.md)에서 확인한다.
