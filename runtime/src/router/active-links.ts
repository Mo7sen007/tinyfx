// @tinyfx/runtime — active link highlighting

/**
 * Mark anchor tags whose `href` pathname matches `pathname` with
 * `data-active="true"`, removing the attribute from all others.
 *
 * Only the pathname portion of each href is compared (query strings and
 * fragments are ignored).
 *
 * @example
 * // In a nav: <a href="/blog">Blog</a>
 * // On the /blog page this element will get data-active="true"
 * highlightActiveLinks("/blog");
 */
export function highlightActiveLinks(pathname: string): void {
  const links = document.querySelectorAll<HTMLAnchorElement>("a[href]");

  links.forEach((link) => {
    try {
      // Resolve href against the current origin to normalise relative URLs
      const url = new URL(link.getAttribute("href")!, window.location.origin);

      if (url.pathname === pathname) {
        link.setAttribute("data-active", "true");
      } else {
        link.removeAttribute("data-active");
      }
    } catch {
      // Ignore malformed / external hrefs
    }
  });
}
