ALTER TABLE "stocks" ALTER COLUMN "ticker" TYPE VARCHAR(10);
ALTER TABLE "stocks" ADD COLUMN "volatility" DOUBLE PRECISION NOT NULL;
ALTER TABLE "stocks" ADD COLUMN "model_type" TEXT NOT NULL;
ALTER TABLE "stocks" ADD COLUMN "drift" DOUBLE PRECISION NOT NULL;
ALTER TABLE "stocks" ADD COLUMN "extra_params" JSONB NOT NULL;
ALTER TABLE "stocks" ADD COLUMN "initial_price" DOUBLE PRECISION NOT NULL;
CREATE TABLE "simulations" (
    "id" UUID NOT NULL,
    "stock_id" UUID NOT NULL,
    "time_horizon" DOUBLE PRECISION NOT NULL,
    "steps" BIGINT NOT NULL,
    "random_seed" BIGINT NOT NULL,
    "parameters" JSONB NOT NULL,
    "created_at" TIMESTAMPTZ(6) NOT NULL,
    "updated_at" TIMESTAMPTZ(6) NOT NULL,
    PRIMARY KEY ("id")
);
CREATE INDEX "index_simulations_by_stock_id" ON "simulations" ("stock_id");
