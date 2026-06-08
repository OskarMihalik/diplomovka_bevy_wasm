const fs = require('fs');
const path = require('path');

const srcDir = path.join(__dirname, 'src/controllers');

const files = fs.readdirSync(srcDir).filter(f => f.endsWith('.ts') && f !== 'auth.ts');

for (const file of files) {
    const fullPath = path.join(srcDir, file);
    let content = fs.readFileSync(fullPath, 'utf8');

    // Add imports if missing
    if (!content.includes('jsonOk')) {
        content = `import { jsonOk, jsonEmptyOk, jsonErr, ErrorReason } from '../utils/response';\n` + content;
    }

    // Replace res.json({}) with jsonEmptyOk()
    content = content.replace(/res\.json\(\{\}\)/g, 'jsonEmptyOk(res)');

    // Replace res.json(var) with jsonOk(res, var)
    content = content.replace(/res\.json\(([^)]+)\)/g, 'jsonOk(res, $1)');

    fs.writeFileSync(fullPath, content);
}

