ALTER TABLE "stocks" DROP COLUMN "model_type";
ALTER TABLE "stocks" DROP COLUMN "extra_params";
DROP INDEX "index_simulations_by_stock_id";
ALTER TABLE "simulations" ALTER COLUMN "stock_id" DROP NOT NULL;
ALTER TABLE "simulations" ADD COLUMN "model_type" TEXT NOT NULL;
