import { DeclarationDataCenter } from "./declaration-data.js";

annotateInstances();
annotateInstancesFor()

async function annotateInstances() {
    const dataCenter = await DeclarationDataCenter.init();
    const instanceForLists = [...(document.querySelectorAll(".instances-list"))];

    // for (const instanceForList of instanceForLists) {
    //     const className = instanceForList.id.slice("instances-list-".length);
    //     const instances = await dataCenter.instancesForClass(className);
    //     var innerHTML = "";
    //     for (var instance of instances) {
    //         const instanceLink = dataCenter.declNameToLink(instance);
    //         innerHTML += `<li><a href="${SITE_ROOT}${instanceLink}">${instance}</a></li>`
    //     }
    //     instanceForList.innerHTML = innerHTML;
    // }

    const classNames = instanceForLists.map((instanceForList) => instanceForList.id.slice("instances-list-".length));

    // `Array<Array<Object<String, String>>>`
    const instances = await dataCenter.annotateInstances(classNames);
    for (let i = 0; i < instances.length; i++) {
        let instance = instances[i];
        let instanceForList = instanceForLists[i];
        let innerHTML = "";

        for (let j = 0; j < instance.length; j++) {
            let instanceLink = instance[j].link;
            let instanceName = instance[j].name;
            innerHTML += `<li><a href="${SITE_ROOT}${instanceLink}">${instanceName}</a></li>`
        }

        instanceForList.innerHTML = innerHTML;
    }
}

async function annotateInstancesFor() {
    const dataCenter = await DeclarationDataCenter.init();
    const instanceForLists = [...(document.querySelectorAll(".instances-for-list"))];

    // for (const instanceForList of instanceForLists) {
    //     const typeName = instanceForList.id.slice("instances-for-list-".length);
    //     const instances = await dataCenter.instancesForType(typeName);
    //     var innerHTML = "";
    //     for (var instance of instances) {
    //         const instanceLink = await dataCenter.declNameToLink(instance);
    //         innerHTML += `<li><a href="${SITE_ROOT}${instanceLink}">${instance}</a></li>`
    //     }
    //     instanceForList.innerHTML = innerHTML;
    // }

    const typeNames = instanceForLists.map((instanceForList) => instanceForList.id.slice("instances-for-list-".length));

    // `Array<Array<Object<String, String>>>`
    const instances = await dataCenter.annotateInstancesFor(typeNames);
    for (let i = 0; i < instances.length; i++) {
        let instance = instances[i];
        let instanceForList = instanceForLists[i];
        let innerHTML = "";

        for (let j = 0; j < instance.length; j++) {
            let instanceLink = instance[j].link;
            let instanceName = instance[j].name;
            innerHTML += `<li><a href="${SITE_ROOT}${instanceLink}">${instanceName}</a></li>`
        }

        instanceForList.innerHTML = innerHTML;
    }
}