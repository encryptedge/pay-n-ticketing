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
  "ticket_id" text UNIQUE,
  "email" text,
  "order_id" text UNIQUE,
  "contact_no" text,
  "uni_id" text,
  "uni_name" text,
  "where_you_reside" text,
  "is_paid" bool,
  "ticket_type" text,
  "booked_at" bool,
  "created_at" date
);