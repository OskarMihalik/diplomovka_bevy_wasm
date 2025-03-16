/*
  Warnings:

  - You are about to drop the column `status_id` on the `TagMessage` table. All the data in the column will be lost.

*/
-- DropForeignKey
ALTER TABLE "TagMessage" DROP CONSTRAINT "TagMessage_status_id_fkey";

-- AlterTable
ALTER TABLE "Tag" ADD COLUMN     "status_id" INTEGER;

-- AlterTable
ALTER TABLE "TagMessage" DROP COLUMN "status_id";

-- AddForeignKey
ALTER TABLE "Tag" ADD CONSTRAINT "Tag_status_id_fkey" FOREIGN KEY ("status_id") REFERENCES "Status"("id") ON DELETE SET NULL ON UPDATE CASCADE;
