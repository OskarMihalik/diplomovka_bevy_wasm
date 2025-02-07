/*
  Warnings:

  - Added the required column `created_by_id` to the `Model` table without a default value. This is not possible if the table is not empty.
  - Added the required column `created_by_id` to the `Project` table without a default value. This is not possible if the table is not empty.
  - Added the required column `created_by_id` to the `Tag` table without a default value. This is not possible if the table is not empty.
  - Added the required column `created_by_id` to the `TagMessage` table without a default value. This is not possible if the table is not empty.

*/
-- AlterTable
ALTER TABLE "Model" ADD COLUMN     "created_by_id" INTEGER NOT NULL;

-- AlterTable
ALTER TABLE "Project" ADD COLUMN     "created_by_id" INTEGER NOT NULL;

-- AlterTable
ALTER TABLE "Tag" ADD COLUMN     "created_by_id" INTEGER NOT NULL;

-- AlterTable
ALTER TABLE "TagMessage" ADD COLUMN     "created_by_id" INTEGER NOT NULL;

-- AddForeignKey
ALTER TABLE "Project" ADD CONSTRAINT "Project_created_by_id_fkey" FOREIGN KEY ("created_by_id") REFERENCES "User"("id") ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "Model" ADD CONSTRAINT "Model_created_by_id_fkey" FOREIGN KEY ("created_by_id") REFERENCES "User"("id") ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "Tag" ADD CONSTRAINT "Tag_created_by_id_fkey" FOREIGN KEY ("created_by_id") REFERENCES "User"("id") ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "TagMessage" ADD CONSTRAINT "TagMessage_created_by_id_fkey" FOREIGN KEY ("created_by_id") REFERENCES "User"("id") ON DELETE RESTRICT ON UPDATE CASCADE;
