-- DropForeignKey
ALTER TABLE "TagMessage" DROP CONSTRAINT "TagMessage_tag_id_fkey";

-- AddForeignKey
ALTER TABLE "TagMessage" ADD CONSTRAINT "TagMessage_tag_id_fkey" FOREIGN KEY ("tag_id") REFERENCES "Tag"("id") ON DELETE CASCADE ON UPDATE CASCADE;
