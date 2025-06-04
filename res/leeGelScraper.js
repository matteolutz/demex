const scrapeFilters = () => {
  const allFilters = document.querySelectorAll(
    ".colours-list__tooltip-wrapper > a",
  );

  const filters = [];
  for (const filter of allFilters) {
    const name = filter.innerText;
    const colorStr = filter.style.backgroundColor;
    const colorRegex = /rgb\((\d+),\s*(\d+),\s*(\d+)\)/;

    const match = colorStr.match(colorRegex);
    const r = (parseInt(match[1], 10) / 255).toFixed(4);
    const g = (parseInt(match[2], 10) / 255).toFixed(4);
    const b = (parseInt(match[3], 10) / 255).toFixed(4);

    filters.push(`color_gel!("${name}", [${r}, ${g}, ${b}]),`);
  }

  console.log(filters.join("\n"));
};
