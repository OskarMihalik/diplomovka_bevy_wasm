/*
  Warnings:

  - A unique constraint covering the columns `[title,project_id]` on the table `Status` will be added. If there are existing duplicate values, this will fail.

*/
-- CreateIndex
CREATE UNIQUE INDEX "Status_title_project_id_key" ON "Status"("title", "project_id");
