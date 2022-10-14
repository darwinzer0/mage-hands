<!-- <div class="paper ce-example__content _ce-example__content--small"> -->
<div class="paper"> 
    <div id={editorId}></div>
</div>

<style lang="scss">
    .paper {
        background-color: white;
        color: black;
    }

    .paper :global(h1) {
        color: black;
    }

    .paper :global(h2) {
        color: black;
        font-size: 24px;
    }

    .paper :global(h3) {
        color: black;
    }
    
    .paper :global(h4) {
        color: black;
    }

    .paper :global(h5) {
        color: black;
    }

    .paper :global(h6) {
        color: black;
    }

    /*
    .ce-example__content {
        max-width: 1100px;
        margin: 0 auto;
        -webkit-font-smoothing: antialiased;
        -moz-osx-font-smoothing: grayscale;
    }

    .ce-example__content--small {
        max-width: 500px;
        border-left: 1px solid #222;
        border-right: 1px solid #222;
        padding: 0 15px;
        background-color: white;
    }
    */

    .ce-block__content, .ce-toolbar__content { max-width:calc(100% - 80px) !important; } .cdx-block { max-width: 100% !important; }
</style>

<script>
    /**
     * To initialize the Editor, create a new instance with configuration object
     * @see docs/installation.md for mode details
     */
    import EditorJS from '@editorjs/editorjs';
    import Header from '@editorjs/header';
    import SimpleImage from '@editorjs/simple-image';
    import List from '@editorjs/list';
    import Checklist from '@editorjs/checklist';
    import Quote from '@editorjs/quote';
    import Marker from '@editorjs/marker';
    import CodeTool from '@editorjs/code';
    import Delimiter from '@editorjs/delimiter';
    import InlineCode from '@editorjs/inline-code';
    //import LinkTool from '@editorjs/link';
    import Embed from '@editorjs/embed';
    import Table from '@editorjs/table';

    export let editorId = "editorjs";
    export let data = "";
    export let readOnly = false;
    export let minHeight = 0;

    let editor = new EditorJS({
        /**
         * Enable/Disable the read only mode
         */
        readOnly,

        minHeight,

        /**
         * Wrapper of Editor
        */
        holder: editorId,

        /**
        * Common Inline Toolbar settings
        * - if true (or not specified), the order from 'tool' property will be used
        * - if an array of tool names, this order will be used
        */
        // inlineToolbar: ['link', 'marker', 'bold', 'italic'],
        // inlineToolbar: true,

        /**
        * Tools list
        */
        tools: {
            /**
            * Each Tool is a Plugin. Pass them via 'class' option with necessary settings {@link docs/tools.md}
            */
            header: {
                class: Header,
                inlineToolbar: ['marker', 'link'],
                config: {
                    placeholder: 'Header'
                },
                shortcut: 'CMD+SHIFT+H'
            },

            /**
            * Or pass class directly without any configuration
            */
            image: SimpleImage,

            list: {
                class: List,
                inlineToolbar: true,
                shortcut: 'CMD+SHIFT+L'
            },

            checklist: {
                class: Checklist,
                inlineToolbar: true,
            },

            quote: {
                class: Quote,
                inlineToolbar: true,
                config: {
                    quotePlaceholder: 'Enter a quote',
                    captionPlaceholder: 'Quote\'s author',
                },
                shortcut: 'CMD+SHIFT+O'
            },

            marker: {
                class:  Marker,
                shortcut: 'CMD+SHIFT+M'
            },

            code: {
                class:  CodeTool,
                shortcut: 'CMD+SHIFT+C'
            },

            delimiter: Delimiter,

            inlineCode: {
                class: InlineCode,
                shortcut: 'CMD+SHIFT+C'
            },

            //linkTool: LinkTool,

            embed: Embed,

            table: {
                class: Table,
                inlineToolbar: true,
                shortcut: 'CMD+ALT+T'
            },

        },

        /**
        * This Tool will be used as default
        */
        // defaultBlock: 'paragraph',
        onChange: function(api, event) {
            editor.save().then((savedData) => {
                data = JSON.stringify(savedData);
            });
        }
    });
</script>

