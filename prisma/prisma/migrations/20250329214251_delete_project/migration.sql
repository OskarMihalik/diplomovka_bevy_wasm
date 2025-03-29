-- DropForeignKey
ALTER TABLE "Status" DROP CONSTRAINT "Status_project_id_fkey";

-- AddForeignKey
ALTER TABLE "Status" ADD CONSTRAINT "Status_project_id_fkey" FOREIGN KEY ("project_id") REFERENCES "Project"("id") ON DELETE CASCADE ON UPDATE CASCADE;
