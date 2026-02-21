-- Add collection_public field to user_preference table
DEFINE FIELD collection_public ON TABLE user_preference TYPE bool DEFAULT false;
