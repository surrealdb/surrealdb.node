const fs = require('fs');
const path = require('path');

const sourceDir = './artifacts';
const npmDir = './npm';

const outDirs = fs.readdirSync(npmDir);

const binaries = collectNodeBinaries(sourceDir);
console.log(binaries)

binaries.forEach((file) => {
    // console.log(file);
    // const sourcePath = path.join(sourceDir, file);
    console.log(`moving artifact: ${file}`);
    const terms = file.split('.');
    if (terms.pop() !== 'node') {
        console.error(`non node file found: ${file}`);
        return;
    }
    const platform = terms.pop()
    if (!platform) {
        console.error(`can't find platform for: ${file}`);
        return;
    }

    if (!outDirs.includes(platform)) {
        console.error(`invalid platform: ${platform} for file: ${file}`);
        return;
    }

    const destPath = path.join(npmDir, platform, path.parse(file).base);

    fs.copyFile(file, destPath, (copyErr) => {
        if (copyErr) {
            console.error(`Could not copy the file: ${copyErr}`);
            return;
        }

        console.log(`Copied ${file} to ${destPath}`);
    });
});

function collectNodeBinaries(root) {
    const files = fs.readdirSync(root, { withFileTypes: true })
    const nodeBinaries = files
        .filter((file) => file.isFile() && file.name.endsWith('.node'))
        .map((file) => path.join(root, file.name))

    const dirs = files.filter((file) => file.isDirectory())
    for (const dir of dirs) {
        nodeBinaries.push(...(collectNodeBinaries(path.join(root, dir.name))))
    }
    return nodeBinaries
}