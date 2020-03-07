function load() {
  fetch("http://127.0.0.1:8080/api/v1/products")
    .then(response => {
      return response.json();
    })
    .then(data => {
      render_table(data.list);
    });
}

function render_table(products) {
  var tbody = document.querySelector("#table tbody");
  if (!tbody) {
    throw "Cannot find table";
  }
  for (let i = 0; i < products.length; i++) {
    const product = products[i];
    const row = document.createElement("tr");
    const name = document.createElement("td");
    name.innerText = product["name"];
    const current_stock = document.createElement("td");
    current_stock.innerText = product["current_stock"];
    const max_stock = document.createElement("td");
    max_stock.innerText = product["max_stock"];
    row.appendChild(name);
    row.appendChild(current_stock);
    row.appendChild(max_stock);
    tbody.appendChild(row);
  }
}
