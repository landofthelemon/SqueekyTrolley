function load() {
    const page_size_input = document.getElementById("page_size");
    const page_index_input = document.getElementById("page_index");
    page_size_input.onchange = function() {
        read_values_and_update();
    }
    page_index_input.onchange = function() {
        read_values_and_update();
    }
    document.getElementById("previous").onclick = function() {
        const page_index_input = document.getElementById("page_index");
        page_index_input.value--;
        read_values_and_update();
    }
    document.getElementById("next").onclick = function() {
        const page_index_input = document.getElementById("page_index");
        page_index_input.value++;
        read_values_and_update();
    }
    read_values_and_update();
}

function read_values_and_update() {
    const page_size_input = document.getElementById("page_size");
    const page_index_input = document.getElementById("page_index");
    update_table(page_size_input.value, page_index_input.value);
}

function update_table(page_size, page_index) {
    fetch("http://127.0.0.1:8080/api/v1/products?page_size={page_size}&page_index={page_index}".format({page_size:page_size, page_index:page_index}))
    .then(response => {
      return response.json();
    })
    .then(data => {
      render_table(data);
      //start_web_socket();
    });
}

//Taken from https://stackoverflow.com/questions/610406/javascript-equivalent-to-printf-string-format
String.prototype.format = String.prototype.format ||
function () {
    "use strict";
    var str = this.toString();
    if (arguments.length) {
        var t = typeof arguments[0];
        var key;
        var args = ("string" === t || "number" === t) ?
            Array.prototype.slice.call(arguments)
            : arguments[0];

        for (key in args) {
            str = str.replace(new RegExp("\\{" + key + "\\}", "gi"), args[key]);
        }
    }

    return str;
};

function start_web_socket() {
    if (!window.WebSocket) {
        throw "Browser doesn't support websockets";
    }
    var ws = new WebSocket("ws://localhost:8080/ws");	
    ws.onopen = function() {
       console.log("Connection is open...");
    };
    ws.onmessage = function (evt) { 
       var data = JSON.parse(evt.data);
       render_table(data);
    };
    ws.onclose = function() { 
       console.log("Connection is closed..."); 
    };
}

function render_table(products) {
  var tbody = document.querySelector("#table tbody");
  if (!tbody) {
    throw "Cannot find table";
  }
  while (tbody.firstChild) {
    tbody.removeChild(tbody.firstChild);
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
