export function downloadLink(url: string, filename: string) {
    let div_container = document.createElement("div");
    div_container.style.display = "none";
    let link_element = document.createElement("a");
    link_element.href = url;
    link_element.download = filename;
    div_container.appendChild(link_element);
    document.body.appendChild(div_container);

    link_element.click();

    document.body.removeChild(div_container);
}