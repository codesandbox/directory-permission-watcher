const { Watcher } = require("./library");

const watcher = new Watcher();
watcher.watch(
  {
    directory: "/some-directory",
    excludes: ["node_modules"],
  },
  (changes) => {
    console.log(changes);
  }
);
