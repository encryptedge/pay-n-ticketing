CREATE TABLE "interest" (
  "id" text UNIQUE PRIMARY KEY,
  "name" text,
  "email" text,
  "contact_no" text,
  "uni_id" text,
  "uni_name" text,
  "where_you_reside" text,
  "created_at" date
);

CREATE TABLE "ticket" (
  "id" text UNIQUE PRIMARY KEY,
  "name" text,
  "email" text,
  "contact_no" text,
  "uni_id" text,
  "uni_name" text,
  "where_you_reside" text,
  "id_paid" bool,
  "ticket_type" text,
  "booked_at" bool,
  "created_at" date
);