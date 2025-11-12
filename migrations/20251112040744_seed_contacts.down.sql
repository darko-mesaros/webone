-- Add down migration script here
DELETE FROM contacts WHERE email IN (
  'alice@example.com', 'bob@example.com', 'carol@example.com',
  'david@example.com', 'emma@example.com', 'frank@example.com',
  'grace@example.com', 'henry@example.com', 'iris@example.com',
  'jack@example.com', 'kate@example.com', 'leo@example.com'
);
