/*
  Warnings:

  - You are about to drop the column `model_id` on the `Tag` table. All the data in the column will be lost.
  - Added the required column `project_id` to the `Tag` table without a default value. This is not possible if the table is not empty.

*/
-- DropForeignKey
ALTER TABLE "Tag" DROP CONSTRAINT "Tag_model_id_fkey";

-- AlterTable
ALTER TABLE "Tag" DROP COLUMN "model_id",
ADD COLUMN     "project_id" INTEGER NOT NULL;

-- AddForeignKey
ALTER TABLE "Tag" ADD CONSTRAINT "Tag_project_id_fkey" FOREIGN KEY ("project_id") REFERENCES "Project"("id") ON DELETE RESTRICT ON UPDATE CASCADE;
