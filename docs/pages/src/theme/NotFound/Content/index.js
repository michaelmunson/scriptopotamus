import React from 'react';
import Link from '@docusaurus/Link';

export default function NotFoundContent() {
  return (
    <main className="container margin-vert--xl">
      <div className="row">
        <div className="col col--6 col--offset-3">
          <h1>Page not found</h1>
          <p>
            That page is not in this manual. The <Link to="/">index</Link> lists
            every section.
          </p>
        </div>
      </div>
    </main>
  );
}
