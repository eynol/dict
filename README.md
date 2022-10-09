# dict

在命令行中输入 `dict <你要查询的单词/中文>` 即可查询对应的翻译。

<img width="568" alt="dict_usage" src="https://user-images.githubusercontent.com/12581247/194769969-52cefcab-fa3e-43ff-8a70-f56aa5d067c1.png">

##  如何安装预编译好的文件

1. 在仓库的 release 页面中下载对应平台的压缩包，并解压，得到二进制可执行文件
2. 将二进制文件放到 `$PATH` 中的某个目录中去


## 如何从源码安装

1. 将仓库 clone 到本地
2. 在仓库根目录中执行 `cargo install --path ./` 即可安装到 `～/.Cargo/bin` 目录中
