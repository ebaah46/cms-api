-- Users table (admin accounts)
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    role VARCHAR(50) NOT NULL DEFAULT 'staff',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create index on email for faster lookups
CREATE INDEX idx_users_email ON users(email);

-- Households table
CREATE TABLE households (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    address TEXT,
    phone VARCHAR(50),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Members table
CREATE TYPE gender AS ENUM ('male', 'female', 'unspecified');
CREATE TYPE member_status AS ENUM ('active', 'inactive', 'visitor', 'transferred', 'deceased');
CREATE TABLE members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    first_name VARCHAR(100) NOT NULL,
    last_name VARCHAR(100) NOT NULL,
    email VARCHAR(255),
    phone VARCHAR(50),
    date_of_birth DATE,
    gender gender,
    address TEXT,
    membership_status member_status NOT NULL DEFAULT 'active',
    membership_date DATE,
    household_id UUID REFERENCES households(id) ON DELETE SET NULL,
    household_role VARCHAR(50),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);


-- Enumerations needed for member details
CREATE TYPE marital_status AS ENUM ('single', 'married', 'divorced', 'widowed', 'separated');

CREATE TYPE education_level AS ENUM ('primary', 'jhs', 'shs', 'tetiary', 'none');

-- Members details
CREATE TABLE member_details(
    id UUID DEFAULT gen_random_uuid(),
    member_id UUID NOT NULL REFERENCES members(id) ON DELETE CASCADE,
    place_of_birth VARCHAR(50),
    region_of_birth VARCHAR(50),
    education_level education_level,
    profession VARCHAR(100),
    occupation VARCHAR(100),
    marital_status marital_status NOT NULL DEFAULT 'single',
    spouse_name VARCHAR(100),
    spouse_date_of_birth DATE,
    hometown VARCHAR(100),
    church VARCHAR(100),
    place_of_marriage VARCHAR(50),
    marriage_officiating_minister VARCHAR(100),
    date_of_baptism DATE,
    place_of_baptism VARCHAR(50),
    baptism_officiating_minister VARCHAR(100),
    date_of_confirmation DATE,
    place_of_confirmation VARCHAR(50),
    confirmation_officiating_minister VARCHAR(100),
    confirmation_text VARCHAR(50),
    house_location VARCHAR(50),
    house_number VARCHAR(50),
    gps_address VARCHAR(50)
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- Indexes for members
CREATE INDEX idx_members_household ON members(household_id);
CREATE INDEX idx_members_deleted_at ON members(deleted_at);
CREATE INDEX idx_members_email ON members(email);
CREATE INDEX idx_members_name ON members(last_name, first_name);

-- Groups table
CREATE TABLE groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    group_type VARCHAR(50) NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for group type filtering
CREATE INDEX idx_groups_type ON groups(group_type);

-- Member-Groups join table
CREATE TABLE member_groups (
    member_id UUID NOT NULL REFERENCES members(id) ON DELETE CASCADE,
    group_id UUID NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    role VARCHAR(50) DEFAULT 'member',
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (member_id, group_id)
);

-- Services table (church services/events)
CREATE TABLE services (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    service_date DATE NOT NULL,
    service_time TIME,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for date-based queries
CREATE INDEX idx_services_date ON services(service_date);

-- Attendance table
CREATE TABLE attendance (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    member_id UUID NOT NULL REFERENCES members(id) ON DELETE CASCADE,
    service_id UUID NOT NULL REFERENCES services(id) ON DELETE CASCADE,
    checked_in_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    checked_in_by UUID REFERENCES users(id) ON DELETE SET NULL,
    UNIQUE (member_id, service_id)
);

-- Indexes for attendance queries
CREATE INDEX idx_attendance_member ON attendance(member_id);
CREATE INDEX idx_attendance_service ON attendance(service_id);

-- Refresh tokens table for JWT refresh token management
CREATE TABLE refresh_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    revoked_at TIMESTAMPTZ
);

CREATE INDEX idx_refresh_tokens_user ON refresh_tokens(user_id);
CREATE INDEX idx_refresh_tokens_hash ON refresh_tokens(token_hash);
