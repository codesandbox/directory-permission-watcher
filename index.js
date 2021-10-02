const { Watcher } = require("./library");
const path = require('path');

const watcher = new Watcher();
watcher.watch(
  {
    directory: __dirname,
    excludes: ["node_modules"],
  },
  (err, ...values) => {
    console.log(values);
  }
);
