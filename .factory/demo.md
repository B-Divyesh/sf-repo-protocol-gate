# Demo sandbox

## CLI demo

Run the shipped sample from any directory after installing the binary:

```sh
repo-protocol demo
```

The command creates `repo-protocol-demo-*` under the operating system temporary
directory. It initializes a Git repository, stages a Drizzle migration, its
schema companion, and a matching evidence manifest. It then runs the same
`repo-protocol check --staged` command used in CI. The command prints the
temporary directory so the sample can be inspected. It never reads or writes a
caller repository.

The source sample ships in `cli/examples/demo-repo/` and is compiled into the
binary so the published CLI keeps working without a network connection.

## Browser demo

Open `/demo` or select **Try it with sample data** on the landing page. The
page immediately loads an approved migration with its schema change, approved
generator, required metadata, and matching evidence. The persistent banner
reads **Demo — sample data, nothing is saved**.

Demo session state uses only `sessionStorage` key
`demo:repo-protocol-gate`. It is separate from real use. **Reset demo** restores
the approved migration. **Start for real** removes the demo key and clears the
sample. The form has no submission endpoint and sends no form data off origin.

After the first visit, the service worker caches the documentation shell and
the `/demo` route so the sample can be reloaded offline.
