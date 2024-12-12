$(function () {

        /**
         * テーブル作成
         * @param settings テーブルの各種設定配置
         */
        $.fn.createTable = function (settings) {
            return createTable(this, settings)
        }

        /**
         * すべてのデータを取得
         */
        $.fn.getData = function () {
            //const $table = this.find(".dataTables_scrollBody>table")
            const $table = this.find("table")
            const datatable = $table.DataTable()
            const data = datatable.data()
            return data
        }

        /**
         * 表格重新绘制
         * 当table在非可见的状态被创建时,导致列宽设置不生效,需要调用一下此函数重新计算一下高度
         */
        $.fn.redraw = function () {
            const $table = this.find("table")
            const datatable = $table.DataTable()
            datatable.columns.adjust().draw()
        }

        /**
         * 選択したデータを取得
         */
        $.fn.getCheckedList = function () {
            //const $table = this.find(".dataTables_scrollBody>table")
            const $table = this.find("table")
            const datatable = $table.DataTable()
            const data = datatable.data()
            const checkedObjs = $table.find("tbody>tr>td>input[type='checkbox']:checked")
            const checkedList = []
            for (let i = 0; i < checkedObjs.length; i++) {
                const value = $(checkedObjs[i]).val()
                const index = parseInt(value)
                checkedList.push({
                    rowNum: index,
                    item: data[index]
                })
            }
            return checkedList
        }

        /**
         * セー,ル値を設定
         * @param row 行番号
         * @param col 列番号
         * @param value 値
         */
        $.fn.setCell = function (row, col, value) {
            //const $table = this.find(".dataTables_scrollBody > table")
            const $table = this.find("table")
            const datatable = $table.DataTable()
            debugger

            //セルを更新
            datatable.cell(row, col + 1).data(value).draw()
        }

        /**
         * 设置整行数据
         * @param index 行番号
         * @param data 行数据
         */
        $.fn.setRow = function (index, data) {
            //const $table = this.find(".dataTables_scrollBody > table")
            const $table = this.find("table")
            const datatable = $table.DataTable()

            //セルを更新
            datatable.row(index).data(data).draw(false)
        }

        /**
         * チェック状態を設定
         * @param option 設定値
         *        checked チェック状態
         *        active 活性状態
         * @param row 行番号
         */
        $.fn.setCheckBox = function (option, row) {
            const $table = this.find("table")
            let $obj = null
            if (row === undefined) {//すべての
                $obj = $table.find("tbody>tr>td>input[type='checkbox']")
            } else {
                $obj = $table.find(`tbody>tr:nth-child(${row + 1})>td>input[type='checkbox']`)
            }

            if (option.checked !== undefined) {//チェック状態を設定
                $obj.prop("checked", option.checked)
            }
            if (option.active !== undefined) {//活性状態を設定
                if (option.active) {//活性状態を設定
                    $obj.removeAttr("disabled")
                } else {
                    $obj.attr("disabled", "")
                }

                //非活性数量
                const checkboxDisabledCount = $table.find("tbody>tr>td>input[type='checkbox'][disabled]").length

                //チェックボックス数量
                const checkboxCount = $table.find("tbody>tr>td>input[type='checkbox']").length

                //すべて選択・非選択のチェックボックス
                //const $headCheck = $table.closest(".dataTables_scroll").find(".dataTables_scrollHead > .dataTables_scrollHeadInner > table > thead > tr > th > input[type = 'checkbox']")
                const $headCheck = $table.find("thead>tr>th>input[type='checkbox']")
                if (checkboxDisabledCount === checkboxCount) {//すべて非活性の場合
                    $headCheck.attr("disabled", "")
                } else {
                    $headCheck.removeAttr("disabled")
                }
            }
        }

        /**
         * データテーブルを作成
         * @param $table table対象
         */
        function createTable($tableDiv, customSettings) {
            if (customSettings.data === undefined) {//データを設定しない場合は、属性「data」から読込
                const data = $tableDiv.attr("data")
                try {
                    customSettings.data = JSON.parse(data)
                } catch {
                    customSettings.data = []
                }
            }
            const $table = redraw($tableDiv)
            //const $table = $tableDiv

            //デフォルト設定
            const defaultSettings = {
                ordering: true,
                searching: false,
                info: false,
                paging: false,
                processing: false,
                lengthChange: false,
                serverSide: false,
                scrollX: false,
                lengthMenu: [10, 20, 30, 50, 100],
                pageLength: 100,
                language: {
                    url: "/plugins/datatable/datatables-zh.json",
                    // sInfo: customSettings.paging ? "_TOTAL_ 件中 _START_ から _END_ まで表示" : "検索結果：_TOTAL_件",
                    // sInfoEmpty: "検索結果：0件",
                    // emptyTable: "対象なし",
                    // show:"dsdg",
                    // search:"搜索",
                    // paginate: {
                    //     first: "首页",
                    //     last: "末页",
                    //     next: "下一页",
                    //     previous: "上一页"
                    // },
                },
                search: {//false：大小文字を検索分ける
                    caseInsensitive: false
                }
            }
            const settings = $.extend(defaultSettings, customSettings)

            if (settings.data.length === 0) {//0件の場合は、特別の処理
                settings.paging = false
                settings.searching = false
                settings.info = false
                settings.csv = false
            }

            let csvDomConf = ""
            if (settings.csv) {// CSVボタンの追加
                csvDomConf = `<"d-inline-block ml-2"B>`
                settings.buttons = [{
                    extend: "csv",
                    text: "CSV",
                    charset: "utf-8",
                    bom: true,
                    extension: ".csv",
                    fieldSeparator: ",",
                    className: "btn btn-primary"
                }]
            }

            let searchDomConf = ""
            if (settings.searching) {//絞り込み条件入力表示の場合
                searchDomConf = `<"d-inline-block"f>`
            }

            let infoDomConf = ""
            if (settings.info) {//ページ情報表示の場合
                infoDomConf = `<"dataTable-info"i>`
            }

            let pageListDomConf = ""
            if (settings.paging) {//ページが表示の場合
                settings.lengthChange = true
                pageListDomConf = `<"mr-4"l>`
            }

            //settings.dom = `<"row mt-2"<"col-6 d-flex"${pageListDomConf}${infoDomConf}><"col-6 text-right"${searchDomConf}${csvDomConf}>><"table-wrapper overflow-auto"t><p>`

            //settings.dom = `<"row mt-2"<"col-6 d-flex"${pageListDomConf}><"col-6 text-right"${searchDomConf}${csvDomConf}>><"table-wrapper"t>${infoDomConf}<p>`
            if (settings.iconBtns !== undefined) {// 操作ボタンがある場合
                let iconTitle = "操作"
                if (settings.iconTitle) {
                    iconTitle = settings.iconTitle
                }
                const iconBtnColumn = {
                    data: null,
                    title: iconTitle,
                    className: "text-center",
                    render: function (_, _, _, meta) {
                        let html = `<div class="text-center text-nowrap" operateDiv>`
                        const makeIconBtnHtml = icon => `<i class="bi ${icon} px-2" role="button" data-icon="${icon}" data-index="${meta.row}" role="button"></i>`
                        Object.keys(settings.iconBtns).forEach(key => {
                            html += makeIconBtnHtml(key)
                        })
                        html += "</div>"
                        return html
                    },
                    orderable: false,
                    width: 60
                }
                settings.columns.push(iconBtnColumn)
            }
            if (settings.checkbox) {// チェック,ボックスがあるの場合
                const checkboxColumn = {
                    data: null,
                    title: `<input type="checkbox" checked="checked">`,
                    render: function (_, _, _, meta) {
                        const html = `<input value="${meta.row}" type="checkbox" class="select-item" checked="checked">`
                        return html
                    },
                    orderable: false
                }
                settings.columns.unshift(checkboxColumn)
            }

            //Jquery.DataTable一列目のソートイコン初期の際に、いつも,表示されるバグ対応
            settings.columns.unshift({title: "", data: null, visible: false})
            settings.drawCallback = function () {
                $table.find("tbody>tr>td>div[operateDiv]>i").on("click", function () {//操作ボタンのイコンのクリックイベント
                    const index = parseInt($(this).data("index"))
                    const item = settings.data[index]
                    const icon = $(this).data("icon")
                    settings.iconBtns[icon](item, index)
                })

                //const $headCheck = $table.closest(".dataTables_scroll").find(".dataTables_scrollHead>.dataTables_scrollHeadInner>table>thead>tr>th>input[type='checkbox']")
                const $headCheck = $table.find("thead>tr>th>input[type='checkbox']")
                $headCheck.change(function () {//全選択/取消のクリックエベント
                    const isCheckAll = $(this).is(":checked")
                    $table.find("tbody>tr>td>input[type='checkbox']").prop("checked", isCheckAll)
                    if (settings.onCheckChange) {//チェック状態が変化エベント
                        const checkedList = $table.getCheckedList()
                        settings.onCheckChange(checkedList)
                    }
                })
                $table.find("tbody>tr>td>input[type='checkbox']").change(function () {//選択/取消のクリックエベント
                    const isNotCheckedAll = $table.find("tbody>tr>td>input[type='checkbox']:unchecked").length > 0
                    $headCheck.prop("checked", !isNotCheckedAll)
                    if (settings.onCheckChange) {//チェック状態が変化エベント
                        const checkedList = $table.getCheckedList()
                        settings.onCheckChange(checkedList)
                    }
                },)
            }
            const datatable = $table.DataTable(settings)
            return datatable
        }

        /**
         * 毎回描画
         * @param $tableDiv 対象DIV
         * @return テーブル対象
         */
        function redraw($tableDiv) {

            //既に存在しているテーブ,ルを削除
            $tableDiv.empty()
            //$tableDiv.addClass("no-gutters-row")
            const tableHtml = `<table class="table table-bordered w-100" style="padding-left:-15px;"></table>`
            $tableDiv.append(tableHtml)
            return $tableDiv.find("table")
        }
    }
)