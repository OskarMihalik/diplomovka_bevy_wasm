-- DropForeignKey
ALTER TABLE "TagMessage" DROP CONSTRAINT "TagMessage_status_id_fkey";

-- AlterTable
ALTER TABLE "TagMessage" ALTER COLUMN "status_id" DROP NOT NULL;

-- AddForeignKey
ALTER TABLE "TagMessage" ADD CONSTRAINT "TagMessage_status_id_fkey" FOREIGN KEY ("status_id") REFERENCES "Status"("id") ON DELETE SET NULL ON UPDATE CASCADE;
