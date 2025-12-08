-- Add up migration script here
CREATE TABLE user_profiles (
     user_id TEXT PRIMARY KEY NOT NULL,
     profile_id TEXT NOT NULL
);

CREATE TABLE guild_profiles (
      guild_id TEXT PRIMARY KEY NOT NULL,
      profile_id TEXT NOT NULL
);