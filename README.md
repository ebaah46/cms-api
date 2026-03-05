# Church Management System API

A REST API for managing church members, households, groups, services, and attendance. Built with Rust, Axum, and PostgreSQL.

## Tech Stack

- **Framework**: Axum 0.7
- **Database**: PostgreSQL 16
- **ORM**: SQLx (compile-time checked queries)
- **Authentication**: JWT with refresh tokens
- **Password Hashing**: Argon2

## Project Structure

```
cms-api/
├── src/
│   ├── main.rs              # Entry point
│   ├── lib.rs               # Module exports, AppState
│   ├── config.rs            # Environment configuration
│   ├── errors.rs            # Unified error handling
│   ├── routes/              # HTTP handlers
│   │   ├── auth.rs          # Login, refresh, logout
│   │   ├── users.rs         # Admin user management
│   │   ├── members.rs       # Church member CRUD + CSV import
│   │   ├── households.rs    # Household CRUD + member linking
│   │   ├── groups.rs        # Groups/ministries CRUD
│   │   ├── services.rs      # Church services CRUD
│   │   └── attendance.rs    # Check-in, bulk check-in
│   ├── services/            # Business logic layer
│   ├── repositories/        # Database queries
│   ├── models/              # Database models
│   ├── dto/                 # Request/Response types
│   └── middleware/          # Auth extractors
├── migrations/              # SQLx database migrations
├── tests/                   # Integration tests
├── Dockerfile               # Production build
├── Dockerfile.dev           # Development with hot reload
├── docker-compose.yml       # Production setup
├── docker-compose.dev.yml   # Development setup
└── docker-compose.test.yml  # Test setup
```

## Features

### MVP Entities
- **Users**: Admin accounts with roles (super_admin, admin, staff)
- **Members**: Church members with soft delete
- **Households**: Family groupings with member linking
- **Groups**: Ministries, committees, cell groups (configurable type)
- **Services**: Church services/events
- **Attendance**: Check-in tracking with bulk support

### Authentication
- JWT access tokens (configurable expiration)
- Refresh token rotation
- Role-based access control
- Password hashing with Argon2

### Additional Features
- CSV bulk import for members
- Pagination on all list endpoints
- Search/filter support
- Soft delete for members

## API Endpoints

### Authentication
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/auth/login` | Login, returns tokens |
| POST | `/api/v1/auth/refresh` | Refresh access token |
| POST | `/api/v1/auth/logout` | Revoke refresh token |

### Users (Admin only)
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/users` | List users |
| POST | `/api/v1/users` | Create user |
| GET | `/api/v1/users/:id` | Get user |
| PATCH | `/api/v1/users/:id` | Update user |
| DELETE | `/api/v1/users/:id` | Delete user |

### Members
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/members` | List members (paginated) |
| POST | `/api/v1/members` | Create member |
| POST | `/api/v1/members/import` | Bulk import from CSV |
| GET | `/api/v1/members/:id` | Get member |
| PATCH | `/api/v1/members/:id` | Update member |
| DELETE | `/api/v1/members/:id` | Soft delete member |
| GET | `/api/v1/members/:id/attendance` | Member's attendance |
| GET | `/api/v1/members/:id/groups` | Member's groups |

### Households
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/households` | List households |
| POST | `/api/v1/households` | Create household |
| GET | `/api/v1/households/:id` | Get household |
| PATCH | `/api/v1/households/:id` | Update household |
| DELETE | `/api/v1/households/:id` | Delete household |
| GET | `/api/v1/households/:id/members` | List members |
| PUT | `/api/v1/households/:id/members/:mid` | Link member |
| DELETE | `/api/v1/households/:id/members/:mid` | Unlink member |

### Groups
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/groups` | List groups |
| POST | `/api/v1/groups` | Create group |
| GET | `/api/v1/groups/:id` | Get group |
| PATCH | `/api/v1/groups/:id` | Update group |
| DELETE | `/api/v1/groups/:id` | Delete group |
| GET | `/api/v1/groups/:id/members` | List members |
| POST | `/api/v1/groups/:id/members/:mid` | Add member |
| DELETE | `/api/v1/groups/:id/members/:mid` | Remove member |

### Services
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/services` | List services |
| POST | `/api/v1/services` | Create service |
| GET | `/api/v1/services/:id` | Get service |
| PATCH | `/api/v1/services/:id` | Update service |
| DELETE | `/api/v1/services/:id` | Delete service |
| GET | `/api/v1/services/:id/attendance` | Service attendance |

### Attendance
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/attendance` | Check in member |
| POST | `/api/v1/attendance/bulk` | Bulk check in |
| DELETE | `/api/v1/attendance/:id` | Remove record |

## Getting Started

### Prerequisites
- Rust 1.83+
- PostgreSQL 16+ (or Docker)

### Local Development

1. **Clone and setup:**
   ```bash
   cd cms-api
   cp .env.example .env
   # Edit .env with your database credentials
   ```

2. **Create database:**
   ```bash
   createdb cms_api
   ```

3. **Run the server:**
   ```bash
   cargo run
   ```

### Docker Development

1. **Start everything:**
   ```bash
   docker compose up --build
   ```

2. **With hot reload:**
   ```bash
   docker compose -f docker-compose.dev.yml up --build
   ```

3. **Just the database:**
   ```bash
   docker compose up db
   cargo run
   ```

### Running Tests

1. **Create test database:**
   ```bash
   createdb cms_api_test
   # Or with Docker:
   docker exec -it <container> psql -U postgres -c "CREATE DATABASE cms_api_test"
   ```

2. **Run tests (sequential for DB tests):**
   ```bash
   cargo test -- --test-threads=1
   ```

3. **With output:**
   ```bash
   cargo test -- --test-threads=1 --nocapture
   ```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | Required |
| `JWT_SECRET` | Secret key for JWT signing | Required |
| `JWT_EXPIRATION_HOURS` | Access token lifetime | 24 |
| `REFRESH_TOKEN_EXPIRATION_DAYS` | Refresh token lifetime | 7 |
| `SERVER_HOST` | Server bind address | 0.0.0.0 |
| `SERVER_PORT` | Server port | 3000 |
| `RUST_LOG` | Log level | cms_api=debug |

## CSV Import Format

For bulk member import, use this CSV format:

```csv
first_name,last_name,email,phone,date_of_birth,gender,address,membership_status,membership_date
John,Smith,john@example.com,555-0100,1985-03-15,male,"123 Main St",active,2020-01-01
```

See `sample_members.csv` for a complete example.

## Database Schema

The database includes these tables:
- `users` - Admin accounts
- `households` - Family groupings
- `members` - Church members (with soft delete)
- `groups` - Ministries, committees, etc.
- `member_groups` - Many-to-many relationship
- `services` - Church services/events
- `attendance` - Service attendance records
- `refresh_tokens` - JWT refresh token storage

Migrations are in the `migrations/` directory and run automatically on startup.

## License

Private project - All rights reserved.
