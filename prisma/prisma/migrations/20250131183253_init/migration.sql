/*
  Warnings:

  - A unique constraint covering the columns `[project_id,is_admin]` on the table `ProjectUser` will be added. If there are existing duplicate values, this will fail.
  - Added the required column `is_admin` to the `ProjectUser` table without a default value. This is not possible if the table is not empty.

*/
-- AlterTable
ALTER TABLE "ProjectUser" ADD COLUMN     "is_admin" BOOLEAN NOT NULL;

-- CreateIndex
CREATE UNIQUE INDEX "ProjectUser_project_id_is_admin_key" ON "ProjectUser"("project_id", "is_admin");
