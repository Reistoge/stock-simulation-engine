DROP INDEX "index_stocks_by_profile_id";
CREATE INDEX "index_stocks_by_profile_id" ON "stocks" ("profile_id");
