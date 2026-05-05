-- Add migration script here


-- Enumerations needed for member details
CREATE TYPE marital_status AS ENUM ('single', 'married', 'divorced', 'widowed', 'separated');

CREATE TYPE education_level AS ENUM ('primary', 'jhs', 'shs', 'tetiary', 'none');

-- Members details
CREATE TABLE member_details(
    id UUID DEFAULT gen_random_uuid(),
    member_id UUID NOT NULL REFERENCES members(id) ON DELETE CASCADE,
    communicant BOOLEAN NOT NULL DEFAULT FALSE,
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
    gps_address VARCHAR(50),
    photo_url VARCHAR(50),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);
