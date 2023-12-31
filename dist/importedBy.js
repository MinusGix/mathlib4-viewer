import { DeclarationDataCenter } from "./declaration-data.js";

fillImportedBy();

async function fillImportedBy() {
    if (!MODULE_NAME) {
        return;
    }
    const dataCenter = await DeclarationDataCenter.init();
    const moduleName = MODULE_NAME;
    const importedByList = document.querySelector(".imported-by-list");
    // const importedBy = dataCenter.moduleImportedBy(moduleName);
    // var innerHTML = "";
    // for (var module of importedBy) {
    //     const moduleLink = dataCenter.moduleNameToLink(module);
    //     innerHTML += `<li><a href="${SITE_ROOT}${moduleLink}">${module}</a></li>`
    // }
    // importedByList.innerHTML = innerHTML;

    const importedBy = await dataCenter.linkedImportedBy(moduleName);
    var innerHTML = "";
    for (let i = 0; i < importedBy.length; i++) {
        let module = importedBy[i];
        innerHTML += `<li><a href="${SITE_ROOT}${module.link}">${module.name}</a></li>`;
    }

    importedByList.innerHTML = innerHTML;
}